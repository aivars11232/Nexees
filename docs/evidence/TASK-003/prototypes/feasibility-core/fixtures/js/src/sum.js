// Runnable code fixture for the on-device test backend. The loop bound is
// wrong on purpose: it skips the last value, so the test fails until repaired.
function sum(values) {
  let total = 0;
  for (let i = 0; i < values.length - 1; i++) {
    total += values[i];
  }
  return total;
}
