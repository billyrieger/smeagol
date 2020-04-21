// pub const fn step(cells: Bool8x8, rule: Rule) -> Bool8x8 {
//     let sums = Adder::new()
//         .add(cells.up(1))
//         .add(cells.down(1))
//         .add(cells.left(1))
//         .add(cells.right(1))
//         .add(cells.up(1).left(1))
//         .add(cells.up(1).right(1))
//         .add(cells.down(1).left(1))
//         .add(cells.left(1).right(1))
//         .sum();

//     let alive = cells;
//     let dead = cells.not();

//     Bool8x8::all_false()
//         .or(sums[0].and(rule.birth[0]).and(dead))
//         .or(sums[1].and(rule.birth[1]).and(dead))
//         .or(sums[2].and(rule.birth[2]).and(dead))
//         .or(sums[3].and(rule.birth[3]).and(dead))
//         .or(sums[4].and(rule.birth[4]).and(dead))
//         .or(sums[5].and(rule.birth[5]).and(dead))
//         .or(sums[6].and(rule.birth[6]).and(dead))
//         .or(sums[7].and(rule.birth[7]).and(dead))
//         .or(sums[8].and(rule.birth[8]).and(dead))
//         .or(sums[0].and(rule.survival[0]).and(alive))
//         .or(sums[1].and(rule.survival[1]).and(alive))
//         .or(sums[2].and(rule.survival[2]).and(alive))
//         .or(sums[3].and(rule.survival[3]).and(alive))
//         .or(sums[4].and(rule.survival[4]).and(alive))
//         .or(sums[5].and(rule.survival[5]).and(alive))
//         .or(sums[6].and(rule.survival[6]).and(alive))
//         .or(sums[7].and(rule.survival[7]).and(alive))
//         .or(sums[8].and(rule.survival[8]).and(alive))
// }

// pub const fn step2(cells: Bool8x8, rule: Rule) -> Bool8x8 {
//     let center = Bool8x8(0x_00_00_3C_3C_3C_3C_00_00);
//     step(step(cells, rule), rule).and(center)
// }


// pub const fn evolve(nw: Bool8x8, ne: Bool8x8, sw: Bool8x8, se: Bool8x8, rule:
// Rule) -> Bool8x8 {     // +---------------------------------+
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . a a a a a a|b b . . . . . . |
//     // | . . a a a a a a|b b . . . . . . |
//     // | . . a a a a a a|b b . . . . . . |
//     // | . . a a a a a a|b b . . . . . . |
//     // | . . a a a a a a|b b . . . . . . |
//     // |_.̲_.̲_a̲_a̲_a̲_a̲_a̲_a̲|̲b̲_b̲_.̲_.̲_.̲_.̲_.̲_.̲_|
//     // | . . c c c c c c|d d . . . . . . |
//     // | . . c c c c c c|d d . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // +---------------------------------+
//     let mask_a = Bool8x8(0x_FC_FC_FC_FC_FC_FC_00_00);
//     let mask_b = Bool8x8(0x_03_03_03_03_03_03_00_00);
//     let mask_c = Bool8x8(0x_00_00_00_00_00_00_FC_FC);
//     let mask_d = Bool8x8(0x_00_00_00_00_00_00_03_03);

//     let a = nw.up(2).left(2).and(mask_a);
//     let b = ne.up(2).right(6).and(mask_b);
//     let c = ne.up(2).right(6).and(mask_c);
//     let d = sw.down(6).left(2).and(mask_d);

//     let w = Bool8x8::all_false().or(a).or(b).or(c).or(d).step2(rule);

//     // +---------------------------------+
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . a a|b b b b b b . . |
//     // | . . . . . . a a|b b b b b b . . |
//     // | . . . . . . a a|b b b b b b . . |
//     // | . . . . . . a a|b b b b b b . . |
//     // | . . . . . . a a|b b b b b b . . |
//     // |_.̲_.̲_.̲_.̲_.̲_.̲_a̲_a̲|̲b̲_b̲_b̲_b̲_b̲_b̲_.̲_.̲_|
//     // | . . . . . . c c|d d d d d d . . |
//     // | . . . . . . c c|d d d d d d . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // +---------------------------------+
//     let mask_a = Bool8x8(0x_C0_C0_C0_C0_C0_C0_00_00);
//     let mask_b = Bool8x8(0x_3F_3F_3F_3F_3F_3F_00_00);
//     let mask_c = Bool8x8(0x_00_00_00_00_00_00_C0_C0);
//     let mask_d = Bool8x8(0x_00_00_00_00_00_00_CF_F0);

//     let a = nw.up(2).left(6).and(mask_a);
//     let b = ne.up(2).right(2).and(mask_b);
//     let c = sw.down(6).left(6).and(mask_c);
//     let d = se.down(6).right(2).and(mask_d);

//     let x = Bool8x8::all_false().or(a).or(b).or(c).or(d).step2(rule);

//     // +---------------------------------+
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . a a a a a a|b b . . . . . . |
//     // |_.̲_.̲_a̲_a̲_a̲_a̲_a̲_a̲|̲b̲_b̲_.̲_.̲_.̲_.̲_.̲_.̲_|
//     // | . . c c c c c c|d d . . . . . . |
//     // | . . c c c c c c|d d . . . . . . |
//     // | . . c c c c c c|d d . . . . . . |
//     // | . . c c c c c c|d d . . . . . . |
//     // | . . c c c c c c|d d . . . . . . |
//     // | . . c c c c c c|d d . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // +---------------------------------+
//     let mask_a = Bool8x8(0x_FC_FC_00_00_00_00_00_00);
//     let mask_b = Bool8x8(0x_03_03_00_00_00_00_00_00);
//     let mask_c = Bool8x8(0x_00_00_FC_FC_FC_FC_FC_FC);
//     let mask_d = Bool8x8(0x_00_00_03_03_03_03_03_03);

//     let a = nw.up(6).left(2).and(mask_a);
//     let b = ne.up(6).right(6).and(mask_b);
//     let c = sw.down(2).left(2).and(mask_c);
//     let d = se.down(2).right(6).and(mask_d);

//     let y = Bool8x8::all_false().or(a).or(b).or(c).or(d).step2(rule);

//     // +---------------------------------+
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . a a|b b b b b b . . |
//     // |_.̲_.̲_.̲_.̲_.̲_.̲_a̲_a̲|̲b̲_b̲_b̲_b̲_b̲_b̲_.̲_.̲_|
//     // | . . . . . . c c|d d d d d d . . |
//     // | . . . . . . c c|d d d d d d . . |
//     // | . . . . . . c c|d d d d d d . . |
//     // | . . . . . . c c|d d d d d d . . |
//     // | . . . . . . c c|d d d d d d . . |
//     // | . . . . . . c c|d d d d d d . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // | . . . . . . . .|. . . . . . . . |
//     // +---------------------------------+
//     let mask_a = Bool8x8(0x_C0_C0_00_00_00_00_00_00);
//     let mask_b = Bool8x8(0x_3F_3F_00_00_00_00_00_00);
//     let mask_c = Bool8x8(0x_00_00_C0_C0_C0_C0_C0_C0);
//     let mask_d = Bool8x8(0x_00_00_3F_3F_3F_3F_3F_3F);

//     let a = nw.up(6).left(6).and(mask_a);
//     let b = ne.up(6).right(2).and(mask_b);
//     let c = sw.down(2).left(6).and(mask_c);
//     let d = se.down(2).right(2).and(mask_d);

//     let z = Bool8x8::all_false().or(a).or(b).or(c).or(d).step2(rule);

//     // +-----------------+
//     // | w w w w x x x x |
//     // | w w w w x x x x |
//     // | w w w w x x x x |
//     // | w w w w x x x x |
//     // | y y y y z z z z |
//     // | y y y y z z z z |
//     // | y y y y z z z z |
//     // | y y y y z z z z |
//     // +-----------------+
//     Bool8x8::all_false()
//         .or(w.up(2).left(2))
//         .or(x.up(2).right(2))
//         .or(y.down(2).left(2))
//         .or(z.down(2).right(2))
// }
