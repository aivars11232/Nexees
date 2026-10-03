test("empty list", () => assertEqual(sum([]), 0));
test("three values", () => assertEqual(sum([1, 2, 3]), 6));
test("negative values", () => assertEqual(sum([5, -2]), 3));
