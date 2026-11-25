import assert from "node:assert/strict";
import { test } from "node:test";
import { judge, median, minimum, onCi } from "../lib/lighthouse.ts";

test("the score is the middle of the runs", () => {
  assert.equal(median([98, 92, 90]), 92);
  assert.equal(median([90, 98, 97]), 97);
});

test("a low performance score on a busy machine is a note, not a failure", () => {
  // The runs that failed scripts/task check at a load of 22 on 10 cores.
  assert.equal(judge("performance", median([98, 92, 90]), false), "note");
});

test("a low performance score fails on CI", () => {
  assert.equal(judge("performance", 92, true), "fail");
});

test("a low accessibility score fails everywhere", () => {
  assert.equal(judge("accessibility", 90, false), "fail");
  assert.equal(judge("accessibility", 90, true), "fail");
});

test("a score at the minimum passes", () => {
  assert.equal(judge("performance", minimum, true), "pass");
  assert.equal(judge("accessibility", minimum, false), "pass");
});

test("CI is read from the CI variable", () => {
  assert.equal(onCi({ CI: "true" }), true);
  assert.equal(onCi({}), false);
  assert.equal(onCi({ CI: "" }), false);
  assert.equal(onCi({ CI: "false" }), false);
});
