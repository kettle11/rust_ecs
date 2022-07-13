use rust_ecs::*;

#[derive(Clone)]
struct A(usize);
impl ComponentTrait for A {
    fn clone_vec(data: &[Self]) -> Option<Vec<Self>> {
        Some(data.into())
    }
}
#[test]
fn spawn() {
    let mut world = World::new();
    world.spawn(A(10));
}

#[test]
fn despawn() {
    let mut world = World::new();
    let entity = world.spawn(A(10));
    world.despawn(entity).unwrap();
}

#[test]
fn query_mut() {
    let mut world = World::new();
    let entity = world.spawn(A(3));
    let mut query = world.query::<All<&mut A>>();
    // assert_eq!(query.archetypes_len(), 1);
    for v in query.iter_mut() {
        println!("HI: {:?}", v.0);
    }

    let mut query_b = world.query::<All<&mut A>>();

    for v in query.iter_mut() {
        println!("HI: {:?}", v.0);
    }
}
