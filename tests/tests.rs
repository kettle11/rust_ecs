use rust_ecs::*;

#[derive(Clone)]
struct A;
impl ComponentTrait for A {
    fn clone_vec(data: &[Self]) -> Option<Vec<Self>> {
        Some(data.into())
    }
}
#[test]
fn spawn() {
    let mut world = World::new();
    world.spawn(A);
}

#[test]
fn despawn() {
    let mut world = World::new();
    let entity = world.spawn(A);
    world.despawn(entity).unwrap();
}

#[test]
fn query_mut() {
    let mut world = World::new();
    let entity = world.spawn(A);
    let query = world.query_mut::<All<&A>>();
    assert_eq!(query.archetypes_len(), 1);
    for v in &query {
        println!("HI");
    }
}
