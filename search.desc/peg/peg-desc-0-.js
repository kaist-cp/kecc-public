searchState.loadedDescShard("peg", 0, "<code>rust-peg</code> is a simple yet flexible parser generator that …\nType of a single atomic element of the input, for example …\nFailure (furthest failure location is not yet known)\nSuccess, with final location\nA type that can be used as input to a parser.\nA parser input type supporting the <code>[...]</code> syntax.\nA parser input type supporting the <code>&quot;literal&quot;</code> syntax.\nA parser input type supporting the <code>$()</code> syntax.\nThe result type used internally in the parser.\nType of a slice of the input.\nParse error reporting\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nGet the element at <code>pos</code>, or <code>Failed</code> if past end of input.\nGet a slice of input.\nAttempt to match the <code>literal</code> string at <code>pos</code>, returning …\nThe main macro for creating a PEG parser.\nUtilities for <code>str</code> input\nA set of literals or names that failed to match\nA parse failure.\nThe set of literals that failed to match at that position.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nThe furthest position the parser reached in the input …\nIterator of expected literals\nLine and column within a string\nColumn (1-indexed)\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nLine (1-indexed)\nByte offset from start of string (0-indexed)")