use super::{Minified, Preprocessed, RawSource, Result};

pub fn preprocess(src: RawSource) -> Result<Preprocessed> {
    Ok(Preprocessed(
        src.into_inner()
            .split('\n')
            .map(ToString::to_string)
            .collect(),
    ))
}

pub fn minify(preprocessed: Preprocessed) -> Minified {
    Minified(preprocessed.into_inner().join("\n"))
}
