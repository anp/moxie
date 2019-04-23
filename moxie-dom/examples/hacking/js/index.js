console.log("loading wasm...");
import("../crate/pkg").then(module => {
  console.log("module start ended");
});
