(function() {
    var implementors = Object.fromEntries([["ordered_float",[["impl&lt;'a, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;&amp;'a <a class=\"struct\" href=\"ordered_float/struct.OrderedFloat.html\" title=\"struct ordered_float::OrderedFloat\">OrderedFloat</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"ordered_float/struct.OrderedFloat.html\" title=\"struct ordered_float::OrderedFloat\">OrderedFloat</a>&lt;T&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/std/primitive.reference.html\">&amp;'a T</a>&gt;,</div>"],["impl&lt;'a, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/std/primitive.reference.html\">&amp;'a T</a>&gt; for &amp;'a <a class=\"struct\" href=\"ordered_float/struct.OrderedFloat.html\" title=\"struct ordered_float::OrderedFloat\">OrderedFloat</a>&lt;T&gt;<div class=\"where\">where\n    <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/std/primitive.reference.html\">&amp;'a T</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>,</div>"],["impl&lt;'a, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/std/primitive.reference.html\">&amp;'a T</a>&gt; for <a class=\"struct\" href=\"ordered_float/struct.OrderedFloat.html\" title=\"struct ordered_float::OrderedFloat\">OrderedFloat</a>&lt;T&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/std/primitive.reference.html\">&amp;'a T</a>&gt;,</div>"],["impl&lt;'a, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"struct\" href=\"ordered_float/struct.OrderedFloat.html\" title=\"struct ordered_float::OrderedFloat\">OrderedFloat</a>&lt;T&gt;&gt; for &amp;'a <a class=\"struct\" href=\"ordered_float/struct.OrderedFloat.html\" title=\"struct ordered_float::OrderedFloat\">OrderedFloat</a>&lt;T&gt;<div class=\"where\">where\n    <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/std/primitive.reference.html\">&amp;'a T</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;T&gt;,</div>"],["impl&lt;'a, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;T&gt; for &amp;'a <a class=\"struct\" href=\"ordered_float/struct.OrderedFloat.html\" title=\"struct ordered_float::OrderedFloat\">OrderedFloat</a>&lt;T&gt;<div class=\"where\">where\n    <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/std/primitive.reference.html\">&amp;'a T</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;T&gt;,</div>"],["impl&lt;'a, T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a> for &amp;'a <a class=\"struct\" href=\"ordered_float/struct.OrderedFloat.html\" title=\"struct ordered_float::OrderedFloat\">OrderedFloat</a>&lt;T&gt;"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a> for <a class=\"struct\" href=\"ordered_float/struct.OrderedFloat.html\" title=\"struct ordered_float::OrderedFloat\">OrderedFloat</a>&lt;T&gt;"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;T&gt; for <a class=\"struct\" href=\"ordered_float/struct.OrderedFloat.html\" title=\"struct ordered_float::OrderedFloat\">OrderedFloat</a>&lt;T&gt;"],["impl&lt;T: <a class=\"trait\" href=\"ordered_float/trait.FloatCore.html\" title=\"trait ordered_float::FloatCore\">FloatCore</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a> for &amp;<a class=\"struct\" href=\"ordered_float/struct.NotNan.html\" title=\"struct ordered_float::NotNan\">NotNan</a>&lt;T&gt;"],["impl&lt;T: <a class=\"trait\" href=\"ordered_float/trait.FloatCore.html\" title=\"trait ordered_float::FloatCore\">FloatCore</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a> for <a class=\"struct\" href=\"ordered_float/struct.NotNan.html\" title=\"struct ordered_float::NotNan\">NotNan</a>&lt;T&gt;"],["impl&lt;T: <a class=\"trait\" href=\"ordered_float/trait.FloatCore.html\" title=\"trait ordered_float::FloatCore\">FloatCore</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;&amp;<a class=\"struct\" href=\"ordered_float/struct.NotNan.html\" title=\"struct ordered_float::NotNan\">NotNan</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"ordered_float/struct.NotNan.html\" title=\"struct ordered_float::NotNan\">NotNan</a>&lt;T&gt;"],["impl&lt;T: <a class=\"trait\" href=\"ordered_float/trait.FloatCore.html\" title=\"trait ordered_float::FloatCore\">FloatCore</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/std/primitive.reference.html\">&amp;T</a>&gt; for &amp;<a class=\"struct\" href=\"ordered_float/struct.NotNan.html\" title=\"struct ordered_float::NotNan\">NotNan</a>&lt;T&gt;"],["impl&lt;T: <a class=\"trait\" href=\"ordered_float/trait.FloatCore.html\" title=\"trait ordered_float::FloatCore\">FloatCore</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/std/primitive.reference.html\">&amp;T</a>&gt; for <a class=\"struct\" href=\"ordered_float/struct.NotNan.html\" title=\"struct ordered_float::NotNan\">NotNan</a>&lt;T&gt;"],["impl&lt;T: <a class=\"trait\" href=\"ordered_float/trait.FloatCore.html\" title=\"trait ordered_float::FloatCore\">FloatCore</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"struct\" href=\"ordered_float/struct.NotNan.html\" title=\"struct ordered_float::NotNan\">NotNan</a>&lt;T&gt;&gt; for &amp;<a class=\"struct\" href=\"ordered_float/struct.NotNan.html\" title=\"struct ordered_float::NotNan\">NotNan</a>&lt;T&gt;"],["impl&lt;T: <a class=\"trait\" href=\"ordered_float/trait.FloatCore.html\" title=\"trait ordered_float::FloatCore\">FloatCore</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;T&gt; for &amp;<a class=\"struct\" href=\"ordered_float/struct.NotNan.html\" title=\"struct ordered_float::NotNan\">NotNan</a>&lt;T&gt;"],["impl&lt;T: <a class=\"trait\" href=\"ordered_float/trait.FloatCore.html\" title=\"trait ordered_float::FloatCore\">FloatCore</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;T&gt; for <a class=\"struct\" href=\"ordered_float/struct.NotNan.html\" title=\"struct ordered_float::NotNan\">NotNan</a>&lt;T&gt;"]]],["zerocopy",[["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a> for <a class=\"struct\" href=\"zerocopy/byteorder/struct.F32.html\" title=\"struct zerocopy::byteorder::F32\">F32</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a> for <a class=\"struct\" href=\"zerocopy/byteorder/struct.F64.html\" title=\"struct zerocopy::byteorder::F64\">F64</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a> for <a class=\"struct\" href=\"zerocopy/byteorder/struct.I128.html\" title=\"struct zerocopy::byteorder::I128\">I128</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a> for <a class=\"struct\" href=\"zerocopy/byteorder/struct.I16.html\" title=\"struct zerocopy::byteorder::I16\">I16</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a> for <a class=\"struct\" href=\"zerocopy/byteorder/struct.I32.html\" title=\"struct zerocopy::byteorder::I32\">I32</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a> for <a class=\"struct\" href=\"zerocopy/byteorder/struct.I64.html\" title=\"struct zerocopy::byteorder::I64\">I64</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a> for <a class=\"struct\" href=\"zerocopy/byteorder/struct.Isize.html\" title=\"struct zerocopy::byteorder::Isize\">Isize</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a> for <a class=\"struct\" href=\"zerocopy/byteorder/struct.U128.html\" title=\"struct zerocopy::byteorder::U128\">U128</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a> for <a class=\"struct\" href=\"zerocopy/byteorder/struct.U16.html\" title=\"struct zerocopy::byteorder::U16\">U16</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a> for <a class=\"struct\" href=\"zerocopy/byteorder/struct.U32.html\" title=\"struct zerocopy::byteorder::U32\">U32</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a> for <a class=\"struct\" href=\"zerocopy/byteorder/struct.U64.html\" title=\"struct zerocopy::byteorder::U64\">U64</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a> for <a class=\"struct\" href=\"zerocopy/byteorder/struct.Usize.html\" title=\"struct zerocopy::byteorder::Usize\">Usize</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.f32.html\">f32</a>&gt; for <a class=\"struct\" href=\"zerocopy/byteorder/struct.F32.html\" title=\"struct zerocopy::byteorder::F32\">F32</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.f64.html\">f64</a>&gt; for <a class=\"struct\" href=\"zerocopy/byteorder/struct.F64.html\" title=\"struct zerocopy::byteorder::F64\">F64</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.i128.html\">i128</a>&gt; for <a class=\"struct\" href=\"zerocopy/byteorder/struct.I128.html\" title=\"struct zerocopy::byteorder::I128\">I128</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.i16.html\">i16</a>&gt; for <a class=\"struct\" href=\"zerocopy/byteorder/struct.I16.html\" title=\"struct zerocopy::byteorder::I16\">I16</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.i32.html\">i32</a>&gt; for <a class=\"struct\" href=\"zerocopy/byteorder/struct.I32.html\" title=\"struct zerocopy::byteorder::I32\">I32</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.i64.html\">i64</a>&gt; for <a class=\"struct\" href=\"zerocopy/byteorder/struct.I64.html\" title=\"struct zerocopy::byteorder::I64\">I64</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.isize.html\">isize</a>&gt; for <a class=\"struct\" href=\"zerocopy/byteorder/struct.Isize.html\" title=\"struct zerocopy::byteorder::Isize\">Isize</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.u128.html\">u128</a>&gt; for <a class=\"struct\" href=\"zerocopy/byteorder/struct.U128.html\" title=\"struct zerocopy::byteorder::U128\">U128</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.u16.html\">u16</a>&gt; for <a class=\"struct\" href=\"zerocopy/byteorder/struct.U16.html\" title=\"struct zerocopy::byteorder::U16\">U16</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.u32.html\">u32</a>&gt; for <a class=\"struct\" href=\"zerocopy/byteorder/struct.U32.html\" title=\"struct zerocopy::byteorder::U32\">U32</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.u64.html\">u64</a>&gt; for <a class=\"struct\" href=\"zerocopy/byteorder/struct.U64.html\" title=\"struct zerocopy::byteorder::U64\">U64</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"zerocopy/byteorder/struct.Usize.html\" title=\"struct zerocopy::byteorder::Usize\">Usize</a>&lt;O&gt;"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"struct\" href=\"zerocopy/byteorder/struct.F32.html\" title=\"struct zerocopy::byteorder::F32\">F32</a>&lt;O&gt;&gt; for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.f32.html\">f32</a>"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"struct\" href=\"zerocopy/byteorder/struct.F64.html\" title=\"struct zerocopy::byteorder::F64\">F64</a>&lt;O&gt;&gt; for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.f64.html\">f64</a>"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"struct\" href=\"zerocopy/byteorder/struct.I128.html\" title=\"struct zerocopy::byteorder::I128\">I128</a>&lt;O&gt;&gt; for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.i128.html\">i128</a>"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"struct\" href=\"zerocopy/byteorder/struct.I16.html\" title=\"struct zerocopy::byteorder::I16\">I16</a>&lt;O&gt;&gt; for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.i16.html\">i16</a>"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"struct\" href=\"zerocopy/byteorder/struct.I32.html\" title=\"struct zerocopy::byteorder::I32\">I32</a>&lt;O&gt;&gt; for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.i32.html\">i32</a>"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"struct\" href=\"zerocopy/byteorder/struct.I64.html\" title=\"struct zerocopy::byteorder::I64\">I64</a>&lt;O&gt;&gt; for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.i64.html\">i64</a>"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"struct\" href=\"zerocopy/byteorder/struct.Isize.html\" title=\"struct zerocopy::byteorder::Isize\">Isize</a>&lt;O&gt;&gt; for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.isize.html\">isize</a>"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"struct\" href=\"zerocopy/byteorder/struct.U128.html\" title=\"struct zerocopy::byteorder::U128\">U128</a>&lt;O&gt;&gt; for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.u128.html\">u128</a>"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"struct\" href=\"zerocopy/byteorder/struct.U16.html\" title=\"struct zerocopy::byteorder::U16\">U16</a>&lt;O&gt;&gt; for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.u16.html\">u16</a>"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"struct\" href=\"zerocopy/byteorder/struct.U32.html\" title=\"struct zerocopy::byteorder::U32\">U32</a>&lt;O&gt;&gt; for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.u32.html\">u32</a>"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"struct\" href=\"zerocopy/byteorder/struct.U64.html\" title=\"struct zerocopy::byteorder::U64\">U64</a>&lt;O&gt;&gt; for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.u64.html\">u64</a>"],["impl&lt;O: <a class=\"trait\" href=\"zerocopy/byteorder/trait.ByteOrder.html\" title=\"trait zerocopy::byteorder::ByteOrder\">ByteOrder</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;<a class=\"struct\" href=\"zerocopy/byteorder/struct.Usize.html\" title=\"struct zerocopy::byteorder::Usize\">Usize</a>&lt;O&gt;&gt; for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/core/primitive.usize.html\">usize</a>"]]]]);
    if (window.register_implementors) {
        window.register_implementors(implementors);
    } else {
        window.pending_implementors = implementors;
    }
})()
//{"start":57,"fragment_lengths":[8727,17421]}