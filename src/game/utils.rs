use anchor::game::GameResult;
use serde::Deserialize;
use std::fs;
use std::path::Path;

pub fn load_from_ron_file<D>(path: impl AsRef<Path>) -> GameResult<D>
where
    for<'a> D: Deserialize<'a>,
{
    let ron_data = fs::read_to_string(path)?;
    Ok(ron::from_str::<D>(&ron_data)?)
}
