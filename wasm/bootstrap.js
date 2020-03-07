// We need a seam so the WebAssembly can be imported asynchronously
import("./js/index.ts").catch(console.error);