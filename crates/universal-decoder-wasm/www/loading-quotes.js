// Loading quotes and code snippets from legendary programmers
// Displayed during WASM initialization

export const LOADING_QUOTES = [
    {
        text: "Talk is cheap. Show me the code.",
        author: "Linus Torvalds",
        type: "quote"
    },
    {
        text: "The best code is no code at all.",
        author: "Jeff Atwood",
        type: "quote"
    },
    {
        text: "Premature optimization is the root of all evil.",
        author: "Donald Knuth",
        type: "quote"
    },
    {
        text: "There are only two hard things in Computer Science: cache invalidation and naming things.",
        author: "Phil Karlton",
        type: "quote"
    },
    {
        text: "Measuring programming progress by lines of code is like measuring aircraft building progress by weight.",
        author: "Bill Gates",
        type: "quote"
    },
    {
        text: "The sooner you start to code, the longer the program will take.",
        author: "Roy Carlson",
        type: "quote"
    },
    {
        text: "First, solve the problem. Then, write the code.",
        author: "John Johnson",
        type: "quote"
    },
    {
        text: "Any fool can write code that a computer can understand. Good programmers write code that humans can understand.",
        author: "Martin Fowler",
        type: "quote"
    }
];

export const LOADING_CODE_SNIPPETS = [
    {
        code: `// The three virtues of a programmer:
// 1. Laziness
// 2. Impatience
// 3. Hubris
while (compiling) { make_coffee(); }`,
        caption: "Classic developer workflow"
    },
    {
        code: `fn blockchain_decode() -> Result<Tx, Error> {
    // It's not a bug, it's a feature!
    todo!("Waiting for WASM to load...")
}`,
        caption: "Rust developers be like..."
    },
    {
        code: `if (problem.isSolved()) {
    // Perfect! Ship it!
} else {
    // TODO: Fix later
    return null;
}`,
        caption: "Production-ready code™"
    },
    {
        code: `// 99 little bugs in the code
// 99 little bugs in the code
// Take one down, patch it around
// 127 little bugs in the code`,
        caption: "The debugging cycle"
    },
    {
        code: `const blockchain = {
    decode: async () => "Loading...",
    patience: Infinity,
    coffee_consumed: Math.random() * 10
};`,
        caption: "JavaScript: Just waiting things"
    },
    {
        code: `// Carmack's optimization motto:
// If it's stupid but it works,
// it's still stupid and you got lucky.
optimize() || refactor() || ship_anyway();`,
        caption: "Game dev wisdom"
    },
    {
        code: `fn main() {
    println!("Loading WASM...");
    // If you're reading this,
    // you're probably procrastinating
    thread::sleep(Duration::from_secs(3));
}`,
        caption: "Honest comments"
    },
    {
        code: `// Linus Torvalds quote (paraphrased):
// "Bad programmers worry about the code.
//  Good programmers worry about data structures."
struct WASM { loading: bool }`,
        caption: "Data > Code"
    }
];

// Get a random quote
export function getRandomQuote() {
    return LOADING_QUOTES[Math.floor(Math.random() * LOADING_QUOTES.length)];
}

// Get a random code snippet
export function getRandomCodeSnippet() {
    return LOADING_CODE_SNIPPETS[Math.floor(Math.random() * LOADING_CODE_SNIPPETS.length)];
}

// Get random loading message
export function getRandomLoadingMessage() {
    const useQuote = Math.random() > 0.5;
    return useQuote ? getRandomQuote() : getRandomCodeSnippet();
}
