use thiserror::Error;

// `thiserror` is fucking great and will allow us to define our own custom
// errors but handling all the errors from third party crates or APIs all
// nice like. Then we can either impl a nice way to print them here and pass
// them along to our std::out layer OR handle the enum there.

#[derive(Error, Debug)]
enum _LyricsError {
    #[error("Request failed")]
    ReqwestError(reqwest::Error),
}
