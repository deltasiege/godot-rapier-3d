// Builds the project tests as a headless binary for the current platfrom
import { $ } from "bun";
import { getData } from ".";
import { join } from "path";
import { glob } from "glob";
import { basename } from "path";

function getCombos(array: any[]) {
  const combos = [];
  for (let i = 0; i < array.length; i++) {
    for (let j = i + 1; j < array.length; j++) {
      combos.push([array[i], array[j]]);
    }
  }
  return combos;
}

async function main() {
  const { buildDir } = getData();
  const searchPath = join(buildDir, "reports", "**report.txt");
  console.log(`Looking in ${searchPath} for reports`);
  const reports = await glob(searchPath, { windowsPathsNoEscape: true });
  if (!reports.length) {
    console.log("No reports found");
    return;
  }
  console.log("Found reports:");
  console.log(reports.map((path) => basename(path)).join("\n"), "\n");
  if (reports.length < 2) return;

  const combos = getCombos(reports);
  const promises = combos.map(async (pair) => {
    const [report1, report2] = pair;
    const outPath = join(
      buildDir,
      "reports",
      `${getReportName(report1)}-${getReportName(report2)}--comparison.diff`
    );
    const filename1 = basename(report1, ".txt");
    const filename2 = basename(report2, ".txt");

    await $`csplit --quiet --prefix=${filename1}- ${report1} "/step#, rapier hash, godot hash/"`;
    await $`csplit --quiet --prefix=${filename2}- ${report2} "/step#, rapier hash, godot hash/"`;

    const headers1 = `${filename1}-00`;
    const headers2 = `${filename2}-00`;
    const hashes1 = `${filename1}-01`;
    const hashes2 = `${filename2}-01`;

    const hasDiff =
      (
        await $`git --no-pager diff --no-index --exit-code --quiet ${hashes1} ${hashes2}`.nothrow()
      ).exitCode === 1;

    const msgPrefix = `Comparing ${basename(report1)} and ${basename(
      report2
    )} ->`;
    if (hasDiff) {
      console.log(`${msgPrefix} Hash mismatches detected`);
      await $`echo Hash mismatches detected > ${outPath}`.nothrow();
    } else {
      console.log(`${msgPrefix} All hashes matched`);
      await $`echo All hashes matched > ${outPath}`.nothrow();
    }

    console.log(`Writing diff to ${outPath}`);
    await $`git --no-pager diff --no-index --word-diff --color-words --color --unified=1000 ${report1} ${report2} | tail -n +6 | tee -a ${outPath} >/dev/null`.nothrow();
    return $`rm ${headers1} ${headers2} ${hashes1} ${hashes2} 2>/dev/null`.nothrow();
  });

  await Promise.all(promises);
}

function getReportName(path: string) {
  return basename(path).replace(".txt", "");
}

await main();
