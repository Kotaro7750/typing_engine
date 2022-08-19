use crate::query::Query;
use crate::query::QueryRequest;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum TypingEngineState {
    Uninitialized,
    Ready,
    Started,
}

/// The main engine of typing game.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TypingEngine {
    state: TypingEngineState,
    query: Option<Query>,
}

impl TypingEngine {
    /// Construct an empty engine.
    ///
    /// This method only do construct typing engine, so you must call [`init`](Self::init()) method to construct
    /// query and [`start`](Self::start()) method to start typing.
    pub fn new() -> Self {
        Self {
            state: TypingEngineState::Uninitialized,
            query: None,
        }
    }

    /// Construct and reset query using [`QueryRequest`].
    pub fn init(&mut self, query_request: QueryRequest) {
        self.query.replace(query_request.construct_query());
        self.state = TypingEngineState::Ready;
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
