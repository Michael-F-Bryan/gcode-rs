// We need a seam so the WebAssembly can be imported asynchronously
import("./ts/index.ts").catch(console.error);