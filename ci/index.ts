import { resolve, join } from "path";

/*
  Flow:

  build workflow runs
    Build GR3D addon and upload binary artifacts
  
  test workflow runs
    Downloads specific binaries package for runner
    runs bun    
     - downloads godot binary
     - builds godot test project
     - runs godot test project to produce reports
    
    - github action use git diff to compare reports
    - github action uploads report artifact
  
  release workflow runs
    - download and include report in release
*/
export type OSName = "linux" | "windows" | "macos";

// https://doc.rust-lang.org/nightly/rustc/platform-support.html#tier-1-with-host-tools
export const targets = [
  "i686-pc-windows-msvc", // windows 32
  "x86_64-pc-windows-msvc", // windows 64
  "x86_64-apple-darwin", // macos 64
  "aarch64-apple-darwin", // macos arm64
  "i686-unknown-linux-gnu", // linux 32
  "x86_64-unknown-linux-gnu", // linux 64
  "aarch64-unknown-linux-gnu", // linux arm64
] as const;
export type Target = (typeof targets)[number];

export function getData(target?: Target): any {
  const projectDir = resolve(import.meta.dir, "..");
  const data = {
    projectDir,
    buildDir: resolve(projectDir, "build"),
    reportsDir: resolve(projectDir, "build", "reports"),
    ciDir: resolve(projectDir, "ci"),
    ciBinDir: resolve(projectDir, "ci", "bin"),
  };
  if (!target) return data;
  const osName = getOSName(target);
  return {
    ...data,
    osName,
    projectBinExt: getProjectBinExt(target),
    godot: getGodotData(osName),
    gr3d: getGR3DData(target, osName),
    tests: getGR3DTestsData(target),
  };
}

function getOSName(target: Target): OSName {
  const osNames = {
    "i686-pc-windows-msvc": "windows",
    "x86_64-pc-windows-msvc": "windows",
    "x86_64-apple-darwin": "macos",
    "aarch64-apple-darwin": "macos",
    "i686-unknown-linux-gnu": "linux",
    "x86_64-unknown-linux-gnu": "linux",
    "aarch64-unknown-linux-gnu": "linux",
  };
  return osNames[target] as OSName;
}

function getProjectBinExt(target: Target) {
  const binExts = {
    "i686-pc-windows-msvc": "exe",
    "x86_64-pc-windows-msvc": "exe",
    "x86_64-apple-darwin": "zip",
    "aarch64-apple-darwin": "zip",
    "i686-unknown-linux-gnu": "x86_32",
    "x86_64-unknown-linux-gnu": "x86_64",
    "aarch64-unknown-linux-gnu": "arm64",
  };
  return binExts[target];
}

function getGodotData(osName: OSName) {
  const binExt = osName === "windows" ? "win64_console.exe" : "linux.x86_64";
  const packExt =
    osName === "windows"
      ? "win64.exe.zip"
      : osName === "macos"
      ? "macos.universal.zip"
      : "linux.x86_64.zip";
  const baseUrl = "https://github.com/godotengine/godot/releases/download";
  const version = "4.4-stable";
  return {
    version,
    binary:
      osName === "macos"
        ? "Godot.app/Contents/MacOS/Godot"
        : `Godot_v${version}_${binExt}`,
    url: `${baseUrl}/${version}/Godot_v${version}_${packExt}`,
    exportTemplatesUrl: `${baseUrl}/${version}/Godot_v${version}_export_templates.tpz`,
    exportTemplatesDir: getExportTemplatesDir(osName),
  };
}

function getGR3DData(target: Target, osName: OSName) {
  const binExt =
    osName === "windows" ? "dll" : osName === "macos" ? "dylib" : "so";
  const packExt = osName === "windows" ? "zip" : "tar.gz";
  const baseUrl = "https://github.com/deltasiege/godot-rapier-3d/releases";
  const version = "latest";
  return {
    version,
    binary: `${target}-godot_rapier_3d.${binExt}`,
    url: `${baseUrl}/${version}/download/godot-rapier-3d--${target}.${packExt}`,
  };
}

function getGR3DTestsData(target: Target) {
  const binExts = {
    "i686-pc-windows-msvc": "exe",
    "x86_64-pc-windows-msvc": "exe",
    "x86_64-apple-darwin": "zip",
    "aarch64-apple-darwin": "zip",
    "i686-unknown-linux-gnu": "x86_32",
    "x86_64-unknown-linux-gnu": "x86_64",
    "aarch64-unknown-linux-gnu": "arm64",
  };
  return {
    binary: `gr3d-tests-${target}.${binExts[target]}`,
  };
}

export function getExportTemplatesDir(
  osName: OSName,
  version: string = "4.4.stable"
) {
  const trail =
    osName === "windows"
      ? "/AppData/Roaming/Godot/export_templates"
      : osName === "macos"
      ? "/Library/Application Support/Godot/export_templates"
      : ".local/share/godot/export_templates";

  return join(process.env.HOME || "", trail, version);
}

export function runnerEnvToOSName(env: NodeJS.ProcessEnv): OSName {
  const OSNames: any = {
    Windows: "windows",
    macOS: "macos",
    Linux: "linux",
  };
  return OSNames[process.env.RUNNER_OS || ""] as OSName;
}
