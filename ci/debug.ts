// Prints debug info to terminal
import { $ } from "bun";
import { getExportTemplatesDir, runnerEnvToOSName } from ".";

async function main() {
  const paths = [
    "./",
    "./ci/bin",
    "./addons/godot-rapier-3d/bin",
    "./build",
    "./build/reports",
    getExportTemplatesDir(runnerEnvToOSName(process.env)),
  ];

  printCommand("pwd");
  await $`pwd`;

  for (const path of paths) {
    await ls(path);
  }

  printCommand("cat godot-rapier-3d.gdextension");
  await $`cat godot-rapier-3d.gdextension`;

  printCommand("bun --print process.env");
  await $`bun --print process.env`;
}

async function ls(path: string) {
  printCommand(`ls -la ${path}`);
  await $`ls -la ${path} || true`.nothrow();
}

function printCommand(command: string) {
  console.log("\n---");
  console.log(`$ ${command}`);
  console.log("---");
}

await main();
