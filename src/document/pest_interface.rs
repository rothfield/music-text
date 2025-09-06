use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "document/grammar.pest"]
pub struct MusicParser;

pub use pest::iterators::Pair;
pub use pest::iterators::Pairs;
pub use pest::error::Error;

// The actual parsing function - this is the interface to Pest
pub fn parse(input: &str) -> Result<Pairs<'_, Rule>, Error<Rule>> {
    MusicParser::parse(Rule::document, input)
}