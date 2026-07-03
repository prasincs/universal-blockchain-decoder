#!/usr/bin/env python3
"""Fetch KNOWN on-chain transactions as self-certifying test fixtures.

The existing fixture corpus was partly hand-crafted, and two "real" Ethereum
fixtures turned out to be defective (one unsigned, one with corrupt RLP — see
crates/decoder-ethereum/tests/alloy_differential.rs). This fetcher prevents
that class of corpus rot: every fixture it writes is anchored to a real
transaction id, and the txid is RECOMPUTED LOCALLY from the raw bytes before
anything is written. A fixture that doesn't hash to its claimed txid is
rejected, network response or not. The txid stored in the .json sidecar lets
anyone re-verify the fixture against a block explorer independently.

Stdlib only (includes a self-tested Keccak-256 implementation, since hashlib
has NIST SHA3 but not Keccak). Network egress to an RPC endpoint / Esplora
instance is required; in restricted environments run this elsewhere and
commit the fixtures — verification is repeated offline by the test suite.

Usage:
    scripts/loop/fetch_corpus.py ethereum 0x<txid> --name eth_erc20_transfer \
        [--rpc https://ethereum-rpc.publicnode.com] \
        [--out crates/decoder-ethereum/tests/fixtures]
    scripts/loop/fetch_corpus.py bitcoin <txid> --name btc_taproot_spend \
        [--esplora https://blockstream.info/api] \
        [--out crates/decoder-bitcoin/tests/fixtures]

The Ethereum RPC must support eth_getRawTransactionByHash (most full nodes
and providers do).
"""

import argparse
import datetime
import hashlib
import json
import sys
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent.parent


# ---------------------------------------------------------------------------
# Keccak-256 (the Ethereum variant: pad 0x01, not SHA3's 0x06). Stdlib-only.
# Verified by _keccak_self_test() on every run.
# ---------------------------------------------------------------------------

_ROUND_CONSTANTS = [
    0x0000000000000001, 0x0000000000008082, 0x800000000000808A, 0x8000000080008000,
    0x000000000000808B, 0x0000000080000001, 0x8000000080008081, 0x8000000000008009,
    0x000000000000008A, 0x0000000000000088, 0x0000000080008009, 0x000000008000000A,
    0x000000008000808B, 0x800000000000008B, 0x8000000000008089, 0x8000000000008003,
    0x8000000000008002, 0x8000000000000080, 0x000000000000800A, 0x800000008000000A,
    0x8000000080008081, 0x8000000000008080, 0x0000000080000001, 0x8000000080008008,
]
_ROTATION = [
    [0, 36, 3, 41, 18],
    [1, 44, 10, 45, 2],
    [62, 6, 43, 15, 61],
    [28, 55, 25, 21, 56],
    [27, 20, 39, 8, 14],
]
_MASK = (1 << 64) - 1


def _rotl(value, shift):
    return ((value << shift) | (value >> (64 - shift))) & _MASK


def _keccak_f(state):
    for rc in _ROUND_CONSTANTS:
        # theta
        c = [state[x][0] ^ state[x][1] ^ state[x][2] ^ state[x][3] ^ state[x][4] for x in range(5)]
        d = [c[(x - 1) % 5] ^ _rotl(c[(x + 1) % 5], 1) for x in range(5)]
        for x in range(5):
            for y in range(5):
                state[x][y] ^= d[x]
        # rho + pi
        b = [[0] * 5 for _ in range(5)]
        for x in range(5):
            for y in range(5):
                b[y][(2 * x + 3 * y) % 5] = _rotl(state[x][y], _ROTATION[x][y])
        # chi
        for x in range(5):
            for y in range(5):
                state[x][y] = b[x][y] ^ ((~b[(x + 1) % 5][y]) & b[(x + 2) % 5][y] & _MASK)
        # iota
        state[0][0] ^= rc
    return state


def keccak256(data: bytes) -> bytes:
    rate = 136  # bytes, for 256-bit output
    # pad10*1 with Keccak domain byte 0x01
    padded = bytearray(data)
    pad_len = rate - (len(padded) % rate)
    padded += b"\x01" + b"\x00" * (pad_len - 2) + b"\x80" if pad_len >= 2 else b"\x81"
    state = [[0] * 5 for _ in range(5)]
    for block_start in range(0, len(padded), rate):
        block = padded[block_start : block_start + rate]
        for i in range(rate // 8):
            x, y = i % 5, i // 5
            state[x][y] ^= int.from_bytes(block[i * 8 : i * 8 + 8], "little")
        _keccak_f(state)
    out = bytearray()
    for i in range(4):  # 32 bytes = first 4 lanes
        x, y = i % 5, i // 5
        out += state[x][y].to_bytes(8, "little")
    return bytes(out)


def _keccak_self_test():
    vectors = {
        b"": "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470",
        b"abc": "4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45",
    }
    for msg, expected in vectors.items():
        got = keccak256(msg).hex()
        if got != expected:
            sys.exit(f"keccak256 self-test FAILED for {msg!r}: got {got}, want {expected}")


# ---------------------------------------------------------------------------
# Bitcoin txid: double-SHA256 over the witness-STRIPPED serialization.
# ---------------------------------------------------------------------------


def _read_varint(buf, i):
    v = buf[i]
    if v < 0xFD:
        return v, i + 1
    if v == 0xFD:
        return int.from_bytes(buf[i + 1 : i + 3], "little"), i + 3
    if v == 0xFE:
        return int.from_bytes(buf[i + 1 : i + 5], "little"), i + 5
    return int.from_bytes(buf[i + 1 : i + 9], "little"), i + 9


def strip_witness(raw: bytes) -> bytes:
    """Return the txid serialization (witness removed) of a Bitcoin tx."""
    if len(raw) < 10:
        raise ValueError("transaction too short")
    if not (raw[4] == 0x00 and raw[5] == 0x01):
        return raw  # legacy serialization, nothing to strip
    out = bytearray(raw[:4])  # version
    i = 6  # skip marker + flag
    start_inputs = i
    n_in, i = _read_varint(raw, i)
    for _ in range(n_in):
        i += 36  # prev txid + vout
        script_len, i = _read_varint(raw, i)
        i += script_len
        i += 4  # sequence
    n_out, i = _read_varint(raw, i)
    for _ in range(n_out):
        i += 8  # value
        script_len, i = _read_varint(raw, i)
        i += script_len
    out += raw[start_inputs:i]  # inputs + outputs, varints included
    # skip witness data: per input, a vector of byte vectors
    for _ in range(n_in):
        n_items, i = _read_varint(raw, i)
        for _ in range(n_items):
            item_len, i = _read_varint(raw, i)
            i += item_len
    out += raw[i : i + 4]  # locktime
    if i + 4 != len(raw):
        raise ValueError(f"trailing bytes after locktime ({len(raw) - i - 4})")
    return bytes(out)


def bitcoin_txid(raw: bytes) -> str:
    stripped = strip_witness(raw)
    return hashlib.sha256(hashlib.sha256(stripped).digest()).digest()[::-1].hex()


# ---------------------------------------------------------------------------
# Fetching
# ---------------------------------------------------------------------------


def http_json(url, payload=None):
    headers = {"User-Agent": "universal-blockchain-decoder fetch_corpus"}
    data = None
    if payload is not None:
        data = json.dumps(payload).encode()
        headers["Content-Type"] = "application/json"
    req = urllib.request.Request(url, data=data, headers=headers)
    with urllib.request.urlopen(req, timeout=30) as resp:
        return json.load(resp)


def http_text(url):
    req = urllib.request.Request(
        url, headers={"User-Agent": "universal-blockchain-decoder fetch_corpus"}
    )
    with urllib.request.urlopen(req, timeout=30) as resp:
        return resp.read().decode().strip()


def fetch_ethereum(txid: str, rpc: str) -> bytes:
    txid = txid.lower()
    if not txid.startswith("0x") or len(txid) != 66:
        sys.exit("ethereum txid must be 0x + 64 hex chars")
    resp = http_json(
        rpc,
        {"jsonrpc": "2.0", "id": 1, "method": "eth_getRawTransactionByHash", "params": [txid]},
    )
    raw_hex = resp.get("result")
    if not raw_hex or raw_hex == "0x":
        sys.exit(f"RPC returned no raw tx (does {rpc} support eth_getRawTransactionByHash?)")
    raw = bytes.fromhex(raw_hex[2:])
    computed = "0x" + keccak256(raw).hex()
    if computed != txid:
        sys.exit(f"VERIFICATION FAILED: keccak256(raw) = {computed}, requested {txid}")
    return raw


def fetch_bitcoin(txid: str, esplora: str) -> bytes:
    if len(txid) != 64:
        sys.exit("bitcoin txid must be 64 hex chars")
    raw = bytes.fromhex(http_text(f"{esplora.rstrip('/')}/tx/{txid}/hex"))
    computed = bitcoin_txid(raw)
    if computed != txid.lower():
        sys.exit(f"VERIFICATION FAILED: txid(raw) = {computed}, requested {txid}")
    return raw


def write_fixture(out_dir: Path, name: str, raw: bytes, meta: dict):
    out_dir.mkdir(parents=True, exist_ok=True)
    hex_path = out_dir / f"{name}.hex"
    json_path = out_dir / f"{name}.json"
    if hex_path.exists():
        sys.exit(f"refusing to overwrite existing fixture {hex_path}")
    hex_path.write_text(raw.hex() + "\n")
    json_path.write_text(json.dumps(meta, indent=2) + "\n")
    print(f"wrote {hex_path} ({len(raw)} bytes)")
    print(f"wrote {json_path}")


DEFAULT_OUT = {
    "ethereum": "crates/decoder-ethereum/tests/fixtures",
    "bitcoin": "crates/decoder-bitcoin/tests/fixtures",
}


def main():
    _keccak_self_test()
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("chain", choices=["ethereum", "bitcoin"])
    ap.add_argument("txid", help="transaction id as found on-chain")
    ap.add_argument("--name", required=True, help="fixture file name (without extension)")
    ap.add_argument("--rpc", default="https://ethereum-rpc.publicnode.com")
    ap.add_argument("--esplora", default="https://blockstream.info/api")
    ap.add_argument("--out", help=f"output dir (default: per-chain, {DEFAULT_OUT})")
    args = ap.parse_args()

    if args.chain == "ethereum":
        raw = fetch_ethereum(args.txid, args.rpc)
        source = args.rpc
    else:
        raw = fetch_bitcoin(args.txid, args.esplora)
        source = args.esplora

    out_dir = Path(args.out) if args.out else REPO / DEFAULT_OUT[args.chain]
    write_fixture(
        out_dir,
        args.name,
        raw,
        {
            "chain": args.chain,
            "txid": args.txid,
            "source": source,
            "fetched_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
            "verification": "txid recomputed locally from raw bytes before writing",
        },
    )


if __name__ == "__main__":
    main()
