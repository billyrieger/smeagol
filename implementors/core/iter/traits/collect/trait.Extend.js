(function() {var implementors = {};
implementors["slotmap"] = [{"text":"impl&lt;K:&nbsp;<a class=\"trait\" href=\"slotmap/trait.Key.html\" title=\"trait slotmap::Key\">Key</a>, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/collect/trait.Extend.html\" title=\"trait core::iter::traits::collect::Extend\">Extend</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(</a>K, V<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">)</a>&gt; for <a class=\"struct\" href=\"slotmap/secondary/struct.SecondaryMap.html\" title=\"struct slotmap::secondary::SecondaryMap\">SecondaryMap</a>&lt;K, V&gt;","synthetic":false,"types":["slotmap::secondary::SecondaryMap"]},{"text":"impl&lt;'a, K:&nbsp;<a class=\"trait\" href=\"slotmap/trait.Key.html\" title=\"trait slotmap::Key\">Key</a>, V:&nbsp;'a + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/collect/trait.Extend.html\" title=\"trait core::iter::traits::collect::Extend\">Extend</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(</a>K, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;'a </a>V<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">)</a>&gt; for <a class=\"struct\" href=\"slotmap/secondary/struct.SecondaryMap.html\" title=\"struct slotmap::secondary::SecondaryMap\">SecondaryMap</a>&lt;K, V&gt;","synthetic":false,"types":["slotmap::secondary::SecondaryMap"]},{"text":"impl&lt;K, V, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/collect/trait.Extend.html\" title=\"trait core::iter::traits::collect::Extend\">Extend</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(</a>K, V<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">)</a>&gt; for <a class=\"struct\" href=\"slotmap/sparse_secondary/struct.SparseSecondaryMap.html\" title=\"struct slotmap::sparse_secondary::SparseSecondaryMap\">SparseSecondaryMap</a>&lt;K, V, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;K: <a class=\"trait\" href=\"slotmap/trait.Key.html\" title=\"trait slotmap::Key\">Key</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.BuildHasher.html\" title=\"trait core::hash::BuildHasher\">BuildHasher</a>,&nbsp;</span>","synthetic":false,"types":["slotmap::sparse_secondary::SparseSecondaryMap"]},{"text":"impl&lt;'a, K, V, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/collect/trait.Extend.html\" title=\"trait core::iter::traits::collect::Extend\">Extend</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(</a>K, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;'a </a>V<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">)</a>&gt; for <a class=\"struct\" href=\"slotmap/sparse_secondary/struct.SparseSecondaryMap.html\" title=\"struct slotmap::sparse_secondary::SparseSecondaryMap\">SparseSecondaryMap</a>&lt;K, V, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;K: <a class=\"trait\" href=\"slotmap/trait.Key.html\" title=\"trait slotmap::Key\">Key</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;V: 'a + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.BuildHasher.html\" title=\"trait core::hash::BuildHasher\">BuildHasher</a>,&nbsp;</span>","synthetic":false,"types":["slotmap::sparse_secondary::SparseSecondaryMap"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()