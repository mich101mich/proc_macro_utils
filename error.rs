#![allow(dead_code)]

use proc_macro2::{Span, TokenStream};
use std::fmt::Display;

/// A proc-macro error that can be turned into a compile error. More versatile than `syn::Error`
/// in that it can be used to chain multiple errors together and has some convenience functions.
pub(crate) struct Error(TokenStream);

/// A result type that uses the `Error` type as the error variant
pub(crate) type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create a new error with a message and a span. Note that `span()` does not work reliably
    /// on stable, so `new_spanned` should be preferred in most cases.
    pub fn new(span: Span, message: impl Display) -> Self {
        syn::Error::new(span, message).into()
    }
    /// Create a new error with a message and the spans taken from the tokens
    pub fn new_spanned(tokens: impl quote::ToTokens, message: impl Display) -> Self {
        syn::Error::new_spanned(tokens, message).into()
    }
    /// Create a new error with a message and the spans taken from items in an iterator
    pub fn new_from_spans<T: quote::ToTokens>(
        tokens: impl IntoIterator<Item = T>,
        message: impl Display,
    ) -> Self {
        Self::builder().with_spans(tokens, message).build()
    }

    /// Shorthand for `Err(Error::new(span, message))`, because 99.9% of the time you want to return
    /// an `Err` when you create an error.
    pub fn err<R>(span: Span, message: impl Display) -> Result<R> {
        Err(Self::new(span, message))
    }
    /// Shorthand for `Err(Error::new_spanned(tokens, message))`, because 99.9% of the time you want to return
    /// an `Err` when you create an error.
    pub fn err_spanned<R>(tokens: impl quote::ToTokens, message: impl Display) -> Result<R> {
        Err(Self::new_spanned(tokens, message))
    }
    /// Shorthand for `Err(Error::new_from_spans(tokens, message))`, because 99.9% of the time you want to return
    /// an `Err` when you create an error.
    pub fn err_from_spans<T: quote::ToTokens, R>(
        tokens: impl IntoIterator<Item = T>,
        message: impl Display,
    ) -> Result<R> {
        Err(Self::new_from_spans(tokens, message))
    }

    /// Creates an error builder to chain multiple errors together
    pub fn builder() -> ErrorBuilder {
        ErrorBuilder::new()
    }
}

/// A builder for creating multiple errors at once
pub(crate) struct ErrorBuilder(TokenStream);

impl ErrorBuilder {
    /// Use `Error::builder()` instead
    fn new() -> Self {
        Self(TokenStream::new())
    }

    /// Add an error with a message and a span. Same as `Error::new`
    pub fn with<T: Display>(&mut self, span: Span, message: T) -> &mut Self {
        self.with_error(Error::new(span, message))
    }
    /// Add an error with a message and the spans taken from the tokens. Same as `Error::new_spanned`
    pub fn with_spanned(
        &mut self,
        tokens: impl quote::ToTokens,
        message: impl Display,
    ) -> &mut Self {
        self.with_error(Error::new_spanned(tokens, message))
    }
    /// Add an error with a message and the spans taken from items in an iterator
    pub fn with_spans<T: quote::ToTokens>(
        &mut self,
        tokens: impl IntoIterator<Item = T>,
        message: impl Display,
    ) -> &mut Self {
        tokens
            .into_iter()
            .fold(self, |builder, token| builder.with_spanned(token, &message))
    }
    /// Add an already created error
    pub fn with_error(&mut self, error: impl Into<Error>) -> &mut Self {
        self.0.extend(TokenStream::from(error.into()));
        self
    }
    /// Add an already created error
    pub fn push(&mut self, error: impl Into<Error>) {
        self.with_error(error);
    }

    /// Check if there are any errors
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Build the errors into a single error
    pub fn build(&mut self) -> Error {
        Error(std::mem::take(&mut self.0))
    }
    /// Build the errors into a single error and return it as a result
    pub fn build_err<R>(&mut self) -> Result<R> {
        Err(self.build())
    }
    /// Build the errors into a result if there are any, returning `Ok(())` if there are none.
    /// This function is useful if a block of code may or may not add errors, and you want to
    /// return early if there are any:
    /// ```ignore
    /// let mut error = Error::builder();
    /// for item in items {
    ///     if !process_item(item) {
    ///         error.with_spanned(item, "failed to process item");
    ///     }
    /// }
    /// error.ok_or_build()?;
    /// ```
    pub fn ok_or_build(&mut self) -> Result<()> {
        if self.is_empty() {
            Ok(())
        } else {
            self.build_err()
        }
    }
}

impl From<syn::Error> for Error {
    fn from(err: syn::Error) -> Self {
        Error(err.to_compile_error())
    }
}

impl From<TokenStream> for Error {
    fn from(err: TokenStream) -> Self {
        Error(err)
    }
}

impl From<Error> for TokenStream {
    fn from(err: Error) -> Self {
        err.0
    }
}
impl From<Error> for proc_macro::TokenStream {
    fn from(err: Error) -> Self {
        err.0.into()
    }
}
