pub use super::_entities::genres::*;
use crate::errors::{LibraryError, LibraryResult};
use sea_orm::{
    sea_query::OnConflict, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
};

pub fn normalize(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

impl Model {
    pub async fn find_or_create<C: ConnectionTrait>(
        db: &C,
        name: &str,
    ) -> LibraryResult<Option<Self>> {
        let name = normalize(name);
        if name.is_empty() {
            return Ok(None);
        }
        Entity::insert(ActiveModel {
            name: Set(name.clone()),
            ..Default::default()
        })
        .on_conflict(OnConflict::column(Column::Name).do_nothing().to_owned())
        .exec_without_returning(db)
        .await?;
        let genre = Entity::find()
            .filter(Column::Name.eq(name))
            .one(db)
            .await?
            .ok_or_else(|| {
                LibraryError::Internal(loco_rs::Error::string(
                    "genre missing after conflict-safe creation",
                ))
            })?;
        Ok(Some(genre))
    }
}

#[cfg(test)]
mod tests {
    use super::normalize;

    /// Genre names must use Unicode lowercase and single internal spaces.
    ///
    /// Case: TS-15 normalizes seven names including Rock, Cyrillic, and a blank value.
    ///
    /// Expected: Case variants share a name, Hip  Hop becomes hip hop, and blanks stay empty.
    #[test]
    fn ts15_normalization() {
        for (input, expected) in [
            ("Rock", "rock"),
            ("rock", "rock"),
            (" ROCK ", "rock"),
            ("Hip  Hop", "hip hop"),
            ("Шансон", "шансон"),
            ("шансон", "шансон"),
            ("   ", ""),
        ] {
            assert_eq!(normalize(input), expected);
        }
    }
}
