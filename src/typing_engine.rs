use std::error::Error;
use std::fmt::Display;

use crate::chunk::Chunk;
use crate::chunk::TypedChunk;
use crate::query::QueryRequest;
use crate::vocabulary::VocabularyInfo;

/// Error type returned from [`TypingEngine`].
#[derive(Debug)]
pub struct TypingEngineError {
    kind: TypingEngineErrorKind,
}

impl TypingEngineError {
    fn new(kind: TypingEngineErrorKind) -> Self {
        Self { kind }
    }
}

impl Display for TypingEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl Error for TypingEngineError {}

#[derive(Debug)]
enum TypingEngineErrorKind {
    MustBeInitialized,
}

impl TypingEngineErrorKind {
    fn as_str(&self) -> &'static str {
        use TypingEngineErrorKind::*;

        match *self {
            MustBeInitialized => "not initialized",
        }
    }
}

impl Display for TypingEngineErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

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
    ///
    /// If this method is called before initializing via calling [`init`](Self::init()) method, this
    /// method returns error.
    pub fn append_query(&mut self, query_request: QueryRequest) -> Result<(), TypingEngineError> {
        if self.is_initialized() {
            assert!(self.processed_chunk_info.is_some());
            assert!(self.vocabulary_infos.is_some());

            let (mut vocabulary_infos, chunks) = query_request.construct_query().decompose();

            self.vocabulary_infos
                .as_mut()
                .unwrap()
                .append(&mut vocabulary_infos);

            self.processed_chunk_info
                .as_mut()
                .unwrap()
                .append_chunks(chunks);

            Ok(())
        } else {
            Err(TypingEngineError::new(
                TypingEngineErrorKind::MustBeInitialized,
            ))
        }
    }

    /// Start typing.
    pub fn start(&mut self) -> Result<(), TypingEngineError> {
        unimplemented!();
    }

    fn is_initialized(&self) -> bool {
        match self.state {
            TypingEngineState::Uninitialized => false,
            _ => true,
        }
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

    pub(crate) fn append_chunks(&mut self, mut chunks: Vec<Chunk>) {
        self.unprocessed_chunks.append(&mut chunks);
    }
}
