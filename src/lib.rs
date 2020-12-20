pub mod index;
pub mod selection;
pub mod table;

#[cfg(test)]
mod tests {
    use crate::index::{BooleanIndex, DiscreteIndex, Index, UniqueIndex};
    use crate::selection::Row;
    use crate::table::{Indexer, Query, Selectable, Table};

    #[derive(Debug, Clone)]
    struct Person {
        id: u32,
        first_name: String,
        last_name: String,
        age: u8,
    }

    impl Selectable for Person {
        type Indexer = PersonIndexer;
    }

    struct PersonIndexer {
        by_id: UniqueIndex<Person, u32>,
        by_last_name: DiscreteIndex<Person, String>,
        adults: BooleanIndex<Person>,
    }

    impl Default for PersonIndexer {
        fn default() -> Self {
            PersonIndexer {
                by_id: UniqueIndex::new(|person| person.id),
                by_last_name: DiscreteIndex::new(|person| &person.last_name),
                adults: BooleanIndex::new(|person| person.age >= 18),
            }
        }
    }

    // TODO: this should be trivially derivable
    impl Indexer<Person> for PersonIndexer {
        fn add(&mut self, row: Row<Person>, item: &Person) {
            self.by_id.add(row, item);
            self.by_last_name.add(row, item);
            self.adults.add(row, item);
        }

        fn remove(&mut self, row: Row<Person>, item: &Person) {
            self.by_id.remove(row, item);
            self.by_last_name.remove(row, item);
            self.adults.remove(row, item);
        }
    }

    impl<X> Query<Person, X>
    where
        X: AsRef<Table<Person>>,
    {
        fn by_id(&mut self, id: u32) -> &mut Self {
            self.maybe_only_row(self.indexer().by_id.get(&id))
        }

        fn by_last_name(&mut self, last_name: &str) -> &mut Self {
            self.and(&self.indexer().by_last_name.get(last_name).clone())
        }

        fn adults(&mut self) -> &mut Self {
            self.and(&self.indexer().adults.get().clone())
        }
    }

    fn people() -> Table<Person> {
        let mut table = Table::in_memory();

        table.insert(Person {
            id: 1,
            first_name: "Aleksei".to_string(),
            last_name: "Voronov".to_string(),
            age: 28,
        });

        table.insert(Person {
            id: 2,
            first_name: "Polina".to_string(),
            last_name: "Zhuravleva".to_string(),
            age: 32,
        });

        table.insert(Person {
            id: 3,
            first_name: "Olivia Alekseevna".to_string(),
            last_name: "Zhuravleva".to_string(),
            age: 0,
        });

        table
    }

    #[test]
    fn can_select_adults() {
        let people = people(); // let people be people
        let adults: Vec<_> = people.select().adults().collect();

        assert_eq!(adults.len(), 2);
        assert_eq!(adults[0].id, 1);
        assert_eq!(adults[1].id, 2);
    }

    #[test]
    fn can_select_by_last_name() {
        let people = people();
        let zhuralvevas: Vec<_> = people.select().by_last_name("Zhuravleva").collect();

        assert_eq!(zhuralvevas.len(), 2);
        assert_eq!(zhuralvevas[0].id, 2);
        assert_eq!(zhuralvevas[1].id, 3);
    }

    #[test]
    fn can_select_by_id() {
        let people = people();
        let voronov = people.select().by_id(1).first();

        assert!(voronov.is_some());
        assert_eq!(voronov.unwrap().id, 1);
    }

    #[test]
    fn can_iterate() {
        let people = people();
        let babies: Vec<_> = people.select().iter().filter(|p| p.age < 1).collect();

        assert_eq!(babies.len(), 1);
        assert_eq!(babies[0].id, 3);
    }

    #[test]
    fn updating_saves_new_info() {
        let mut people = people();
        let aleksei = people.select().by_id(1).first().unwrap();
        assert_eq!(aleksei.age, 28);

        people.update().by_id(1).apply(|p| p.age = 29);
        let aleksei = people.select().by_id(1).first().unwrap();
        assert_eq!(aleksei.age, 29);
    }

    #[test]
    fn updating_updates_indexes() {
        let mut people = people();

        let voronov = people.select().by_last_name("Voronov").first().unwrap();
        assert_eq!(voronov.last_name, "Voronov");

        people.update().by_last_name("Voronov").apply(|p| p.last_name = "Smith".to_string());

        let voronov = people.select().by_last_name("Voronov").first();
        let smith = people.select().by_last_name("Smith").first();
        assert!(voronov.is_none());
        assert!(smith.is_some());
    }
}
