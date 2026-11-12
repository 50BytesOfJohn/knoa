//! Writing the columns a pulled patch carries, and only those.

use rusqlite::{Connection, params_from_iter, types::Value};

use crate::Result;

#[derive(Default)]
pub(super) struct Columns(Vec<(&'static str, Value)>);

impl Columns {
    /// Adds the column when the patch has it.
    pub fn add<T: Into<Value>>(&mut self, column: &'static str, value: Option<T>) {
        if let Some(value) = value {
            self.0.push((column, value.into()));
        }
    }

    pub fn update(self, conn: &Connection, table: &str, id: i64) -> Result<()> {
        if self.0.is_empty() {
            return Ok(());
        }
        let set: Vec<String> = self
            .0
            .iter()
            .enumerate()
            .map(|(index, (column, _))| format!("{column} = ?{}", index + 2))
            .collect();
        let sql = format!("UPDATE {table} SET {} WHERE id = ?1", set.join(", "));
        let values = std::iter::once(Value::Integer(id)).chain(self.0.into_iter().map(|(_, v)| v));
        conn.execute(&sql, params_from_iter(values))?;
        Ok(())
    }
}
