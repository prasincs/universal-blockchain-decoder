//! Detailed debugging of BoC parsing against tonlib-core
//!
//! This test file helps debug the BoC parser by comparing our implementation
//! step-by-step against tonlib-core's reference implementation.

use base64::{engine::general_purpose::STANDARD, Engine};

#[test]
fn debug_boc_structure() {
    // Simple BoC from tests
    let boc_base64 = "te6cckEBBAEAOgACATQCAQAAART/APSkE/S88sgLAwBI0wHQ0wMBcbCRW+D6QDBwgBDIywVYzxYh+gLLagHPFsmAQPsAlxCarA==";
    let boc_bytes = STANDARD.decode(boc_base64).expect("Invalid base64");

    println!("=== BoC Structure Analysis ===\n");
    println!("Total size: {} bytes", boc_bytes.len());
    println!("Hex: {}\n", hex::encode(&boc_bytes));

    // Parse header manually
    let magic = u32::from_be_bytes([boc_bytes[0], boc_bytes[1], boc_bytes[2], boc_bytes[3]]);
    println!("Magic: 0x{:08x}", magic);

    let flags = boc_bytes[4];
    println!("Flags: 0x{:02x} = 0b{:08b}", flags, flags);
    println!("  has_idx: {}", (flags & 0x80) >> 7);
    println!("  has_crc32c: {}", (flags & 0x40) >> 6);
    println!("  has_cache_bits: {}", (flags & 0x20) >> 5);
    println!("  size: {}", flags & 0x07);

    let off_bytes = boc_bytes[5] as usize;
    println!("\nOff bytes: {}", off_bytes);

    let cells_count = boc_bytes[6] as usize;
    let roots_count = boc_bytes[7] as usize;
    let absent_count = boc_bytes[8] as usize;
    let tot_cells_size = boc_bytes[9] as usize;

    println!("Cells count: {}", cells_count);
    println!("Roots count: {}", roots_count);
    println!("Absent count: {}", absent_count);
    println!("Total cells size: {} bytes", tot_cells_size);

    let root_list_start = 10;
    let root_list_end = root_list_start + roots_count * off_bytes;
    println!(
        "\nRoot list: {:?}",
        &boc_bytes[root_list_start..root_list_end]
    );

    let cells_data_start = root_list_end;
    let cells_data_end = cells_data_start + tot_cells_size;
    let cells_data = &boc_bytes[cells_data_start..cells_data_end];

    println!("\nCells data section ({} bytes):", cells_data.len());
    println!("Hex: {}", hex::encode(cells_data));

    // Parse cells manually
    let mut pos = 0;

    println!("\n=== Parsing Cells ===\n");
    for i in 0..cells_count {
        println!("Cell {}:", i);

        if pos + 2 > cells_data.len() {
            println!("  ERROR: Not enough data for descriptor");
            break;
        }

        let d1 = cells_data[pos];
        let d2 = cells_data[pos + 1];
        pos += 2;

        println!("  Descriptor: d1=0x{:02x}, d2=0x{:02x}", d1, d2);

        let refs_count = (d1 & 0x07) as usize;
        let is_exotic = (d1 & 0x08) != 0;
        let has_hashes = (d1 & 0x10) != 0;
        let level_mask = (d1 >> 5) & 0x07;

        println!("    refs_count: {}", refs_count);
        println!("    is_exotic: {}", is_exotic);
        println!("    has_hashes: {}", has_hashes);
        println!("    level_mask: {}", level_mask);

        // Calculate bit length from d2
        let bit_len = if d2 == 0 {
            // Full bytes, read next byte
            if pos >= cells_data.len() {
                println!("  ERROR: No size byte");
                break;
            }
            let size_byte = cells_data[pos];
            pos += 1;
            println!("    size_byte: {}", size_byte);
            size_byte as u16 * 8
        } else {
            // d2 encodes: floor(b/8) + ceil(b/8)
            // For now, just use d2 as bits
            d2 as u16
        };

        println!("    bit_len: {} bits", bit_len);

        let data_bytes = bit_len.div_ceil(8) as usize;
        println!("    data_bytes: {}", data_bytes);

        if pos + data_bytes > cells_data.len() {
            println!(
                "  ERROR: Not enough data ({} needed, {} available)",
                data_bytes,
                cells_data.len() - pos
            );
            break;
        }

        let cell_data = &cells_data[pos..pos + data_bytes];
        pos += data_bytes;
        println!("    data: {}", hex::encode(cell_data));

        // Read refs
        if pos + refs_count > cells_data.len() {
            println!("  ERROR: Not enough data for refs");
            break;
        }

        let refs: Vec<u8> = cells_data[pos..pos + refs_count].to_vec();
        pos += refs_count;
        println!("    refs: {:?}", refs);
        println!();
    }

    // Parse with tonlib
    println!("\n=== tonlib-core Parsing ===\n");
    let tonlib_result = tonlib_core::cell::BagOfCells::parse(&boc_bytes);
    match tonlib_result {
        Ok(boc) => {
            println!("✓ tonlib successfully parsed");
            println!("Roots: {}", boc.roots.len());
            for (i, root) in boc.roots.iter().enumerate() {
                println!("  Root {}: {} bits", i, root.bit_len());
            }
        }
        Err(e) => {
            println!("✗ tonlib failed: {:?}", e);
        }
    }
}

#[test]
fn debug_our_parser() {
    use decoder_ton::boc;

    let boc_base64 = "te6cckEBBAEAOgACATQCAQAAART/APSkE/S88sgLAwBI0wHQ0wMBcbCRW+D6QDBwgBDIywVYzxYh+gLLagHPFsmAQPsAlxCarA==";
    let boc_bytes = STANDARD.decode(boc_base64).expect("Invalid base64");

    println!("\n=== Our Parser ===\n");
    let result = boc::parse_boc(&boc_bytes);
    match result {
        Ok(cells) => {
            println!("✓ Successfully parsed {} cells", cells.len());
            for (i, cell) in cells.iter().enumerate() {
                println!(
                    "  Cell {}: {} bits, {} refs, {} data bytes",
                    i,
                    cell.bit_len,
                    cell.refs.len(),
                    cell.data.len()
                );
            }
        }
        Err(e) => {
            println!("✗ Parser failed: {:?}", e);
        }
    }
}
