use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_builder::*;
use diesel::query_dsl::methods::LoadQuery;
use diesel::sql_types::BigInt;
use schemars::JsonSchema;
use serde::Serialize;

pub trait Paginate: Sized {
  fn paginate(self, page: i64) -> Paginated<Self>;
}

impl<T> Paginate for T {
  fn paginate(self, page: i64) -> Paginated<Self> {
    Paginated {
      query: self,
      per_page: DEFAULT_PER_PAGE,
      page,
      offset: (page - 1) * DEFAULT_PER_PAGE,
    }
  }
}

const DEFAULT_PER_PAGE: i64 = 10;

#[derive(Debug, Clone, Copy, QueryId)]
pub struct Paginated<T> {
  query: T,
  page: i64,
  per_page: i64,
  offset: i64,
}

impl<T> Paginated<T> {
  pub fn per_page(self, per_page: i64) -> Self {
    Paginated {
      per_page,
      offset: (self.page - 1) * per_page,
      ..self
    }
  }

  pub fn load_and_count_pages<'a, U>(
    self,
    conn: &mut PgConnection,
  ) -> QueryResult<PaginatedResult<U>>
  where
    Self: LoadQuery<'a, PgConnection, (U, i64)>,
  {
    let per_page = self.per_page;
    let page = self.page;
    let results = self.load::<(U, i64)>(conn)?;
    let count = results.get(0).map(|x| x.1).unwrap_or(0);
    let records = results.into_iter().map(|x| x.0).collect();
    let total_pages = (count as f64 / per_page as f64).ceil() as i64;

    Ok(PaginatedResult {
      records,
      total_pages,
      page,
      per_page,
      count,
    })
  }
}

impl<T: Query> Query for Paginated<T> {
  type SqlType = (T::SqlType, BigInt);
}

impl<T> RunQueryDsl<PgConnection> for Paginated<T> {}

impl<T> QueryFragment<Pg> for Paginated<T>
where
  T: QueryFragment<Pg>,
{
  fn walk_ast<'a>(&'a self, mut out: AstPass<'_, 'a, Pg>) -> QueryResult<()> {
    out.push_sql("SELECT *, COUNT(*) OVER () FROM (");
    self.query.walk_ast(out.reborrow())?;
    out.push_sql(") t LIMIT ");
    out.push_bind_param::<BigInt, _>(&self.per_page)?;
    out.push_sql(" OFFSET ");
    out.push_bind_param::<BigInt, _>(&self.offset)?;
    Ok(())
  }
}

#[derive(Serialize, JsonSchema)]
pub struct PaginatedResult<T> {
  pub records: Vec<T>,
  total_pages: i64,
  per_page: i64,
  page: i64,
  count: i64,
}

impl<T> PaginatedResult<T> {
  pub fn records<U>(&self, records: Vec<U>) -> PaginatedResult<U> {
    PaginatedResult {
      records,
      total_pages: self.total_pages,
      per_page: self.per_page,
      page: self.page,
      count: self.count,
    }
  }
}
