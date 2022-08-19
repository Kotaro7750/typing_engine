use crate::chunk::Chunk;
use crate::chunk::TypedChunk;
use crate::query::QueryRequest;
use crate::vocabulary::VocabularyInfo;

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
    processed_chunk_info: Option<ProcessedChunkInfo>,
    vocabulary_infos: Option<Vec<VocabularyInfo>>,
}

impl TypingEngine {
    /// Construct an empty engine.
    ///
    /// This method only do construct typing engine, so you must call [`init`](Self::init()) method to construct
    /// query and [`start`](Self::start()) method to start typing.
    pub fn new() -> Self {
        Self {
            state: TypingEngineState::Uninitialized,
            processed_chunk_info: None,
            vocabulary_infos: None,
        }
    }

    /// Construct and reset query using [`QueryRequest`].
    pub fn init(&mut self, query_request: QueryRequest) {
        let query = query_request.construct_query();
        let (vocabulary_infos, chunks) = query.decompose();

        self.vocabulary_infos.replace(vocabulary_infos);
        self.processed_chunk_info
            .replace(ProcessedChunkInfo::new(chunks));

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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct ProcessedChunkInfo {
    unprocessed_chunks: Vec<Chunk>,
    inflight_chunk: Option<TypedChunk>,
    confirmed_chunks: Vec<TypedChunk>,
}

impl ProcessedChunkInfo {
    pub(crate) fn new(chunks: Vec<Chunk>) -> Self {
        Self {
            unprocessed_chunks: chunks,
            inflight_chunk: None,
            confirmed_chunks: vec![],
        }
    }
}
