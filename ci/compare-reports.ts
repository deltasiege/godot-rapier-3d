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

  const promises = combos.map((pair) => {
    const [report1, report2] = pair;
    console.log(`Comparing ${basename(report1)} and ${basename(report2)}`);
    const output =
      $`git --no-pager diff --numstat --no-index ${report1} ${report2}`
        .nothrow()
        .text()
        .then((output) => {
          if (output) return { report1, report2, output };
        });
    return output;
  });

  const results = await Promise.all(promises);
  const filtered = results.filter(Boolean);

  console.log("\nResults: ");
  console.log("Differences\tPaths");
  filtered.forEach(async ({ report1, report2, output }: any) => {
    console.log(output.replaceAll("\n", ""));
    const outPath = join(
      buildDir,
      "reports",
      `${getReportName(report1)}-${getReportName(report2)}--comparison.diff`
    );

    console.log(`Writing diff to ${outPath}`);
    await $`git --no-pager diff --color-words --color --no-index --word-diff ${report1} ${report2} > ${outPath}`.nothrow();
  });
}

function getReportName(path: string) {
  return basename(path).replace(".txt", "");
}

await main();
