use bevy::prelude::*;

#[derive(Component)]
pub struct Person {
    name: String,
    age: u32
}

fn main() {
    App::new()
        .add_systems(Startup, setup)
        .add_systems(Update, print_person)
        .run();
}

pub fn setup(mut commands: Commands) {
    commands.spawn(Person {
        name: "Bob".to_string(),
        age: 42
    });
}

pub fn print_person(person: Query<&Person>) {
    for person in person.iter() {
        println!("Name: {}, Age: {}", person.name, person.age);
    }
}