// The rule that check-lighthouse applies to the Lighthouse scores. It lives
// apart from the script so that a test can hold it without a browser.

export const minimum = 95;

export type Verdict = "pass" | "fail" | "note";

// Each score is the median of several runs, as the Lighthouse docs advise.
export function median(scores: readonly number[]): number {
  const sorted = scores.toSorted((a, b) => a - b);
  const middle = sorted[Math.floor(sorted.length / 2)];
  if (middle === undefined) throw new Error("median of no scores");
  return middle;
}

// Performance moves with the load on the machine: total blocking time grows
// when other work holds the cores. The criterion holds it on CI, so on a
// developer machine a low performance score is a note and not a failure.
// Accessibility does not depend on the load, so it fails everywhere.
export function judge(category: string, score: number, ci: boolean): Verdict {
  if (score >= minimum) return "pass";
  return category === "performance" && !ci ? "note" : "fail";
}

// GitHub Actions sets CI=true on every runner.
export function onCi(env: Readonly<Record<string, string | undefined>>): boolean {
  const value = env.CI;
  return value !== undefined && value !== "" && value !== "false" && value !== "0";
}
