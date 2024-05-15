import fs from "fs";
import fsExtra from "fs-extra";
import { glob } from "glob";
import { mkdir, unlink } from "fs/promises";
import crossZip from "cross-zip"; // https://github.com/feross/cross-zip
import { resolve, join, basename } from "path";
import { targets, type Target } from ".";

export async function downloadAndExtract(url: string, destDir: string) {
  if (!url || !destDir)
    throw new Error("Invalid downloadAndExtract parameters");
  const fileName = getFileNameFromUrl(url);
  const destPath = resolve(destDir, fileName);
  await downloadIfNotExists(url, destDir);
  await extract(destPath, destDir).catch(console.error);
  return fileName;
}

export function validTarget(target: Target) {
  const valid = targets.includes(target);
  if (!valid)
    console.error(
      "Invalid target passed: '",
      target,
      "'\nValid targets are: ",
      targets.join(", ")
    );
  return valid;
}

export async function extract(from: string, to: string): Promise<void | Error> {
  return new Promise((resolve, reject) => {
    if (!from || !to) reject("Invalid extract parameters");
    console.log(`Extracting ${from} to ${to}`);
    crossZip.unzip(from, to, (err) => {
      if (err) reject(err);
      resolve();
    });
  });
}

export async function downloadToPath(url: string, path: string) {
  if (!url) return;
  console.log(`Downloading ${url} to ${path}`);
  const response = await fetch(url);
  await Bun.write(path, await response.arrayBuffer());
}

export function getFileNameFromUrl(url: string) {
  const fileName = url.split("/").pop();
  if (!fileName)
    throw new Error(`Couldn't determine fileName name from url: ${url}`);
  return fileName;
}

export async function downloadToDir(url: string, dir: string) {
  const fileName = getFileNameFromUrl(url);
  const path = resolve(dir, fileName);
  await downloadToPath(url, path);
  return fileName;
}

export async function downloadIfNotExists(url: string, dir: string) {
  if (!url) return;
  const fileName = getFileNameFromUrl(url);
  const path = resolve(dir, fileName);
  const isDownloaded = await Bun.file(path).exists();
  if (!isDownloaded) await downloadToPath(url, path);
  else console.log(`${path} already downloaded`);
  return fileName;
}

export async function createDir(path: string) {
  if (!path) return;
  if (fs.existsSync(path)) return console.log(`${path} already exists`);
  console.log(`Creating directory ${path}`);
  await mkdir(path, { recursive: true });
}

export async function globMove(from: string, dir: string) {
  console.log(`Moving ${from} to ${dir}`);
  const paths: string[] = await glob(from, {
    dot: true,
    windowsPathsNoEscape: true,
    nodir: true,
  });
  console.log("Globbed paths: ", paths);
  const promises = paths.map((path) => {
    const dest = join(dir, basename(path));
    console.log(`Moving ${path} to ${dest}`);
    return fsExtra.move(path, dest, { overwrite: true });
  });
  return await Promise.all(promises);
}

export async function globRename(from: string, to: string) {
  console.log(`Renaming ${from} to ${to}`);
  const paths: string[] = await glob(from, {
    dot: true,
    windowsPathsNoEscape: true,
    nodir: true,
  });
  console.log("Globbed paths: ", paths);
  const promises = paths.map((path) => {
    console.log(`Renaming ${path} to ${to}`);
    return fsExtra.move(path, to, { overwrite: true });
  });
  return await Promise.all(promises);
}

export async function renameFile(from: string, to: string) {
  await copyFile(from, to);
  await deleteFile(from);
}

export async function copyFile(from: string, to: string) {
  const original = Bun.file(from);
  if (!original) throw new Error(`File not found: ${original}`);
  console.log(`Copying ${from} to ${to}`);
  await Bun.write(to, original);
}

export async function deleteFile(path: string) {
  await unlink(path);
}
