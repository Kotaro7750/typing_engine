use query::QueryRequest;

mod chunk;
mod chunk_key_stroke_dictionary;
mod key_stroke;
mod query;
mod spell;
mod utility;
mod vocabulary;

#[cfg(test)]
mod test_utility;

/// The main engine of typing game.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TypingEngine {}

impl TypingEngine {
    /// Construct an empty engine.
    ///
    /// This method only do construct typing engine, so you must call [`init`](Self::init()) method to construct
    /// query and [`start`](Self::start()) method to start typing.
    pub fn new() -> Self {
        Self {}
    }

    /// Construct query using [`QueryRequest`].
    pub fn init(&mut self, query_request: QueryRequest) {
        unimplemented!();
    }

    /// Append query using [`QueryRequest`].
    pub fn append_query(&mut self, query_request: QueryRequest) {
        unimplemented!();
    }

    /// Start typing.
    pub fn start(&mut self) {
        unimplemented!();
    }
}
