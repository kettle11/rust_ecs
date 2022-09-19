use rust_ecs::*;

#[derive(Clone)]
struct A(usize);
impl ComponentTrait for A {
    fn clone_vec(data: &[Self]) -> Option<Vec<Self>> {
        Some(data.into())
    }
}

#[derive(Clone)]
struct B(usize);
impl ComponentTrait for B {
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
    let _entity = world.spawn(A(3));
    {
        let mut query = world.query::<All<&mut A>>();
        // assert_eq!(query.archetypes_len(), 1);
        for _v in query.iter_mut() {}
    }

    // let mut query_b = world.query_mut::<All<&mut A>>();
}

#[test]
fn query_multiple() {
    let mut world = World::new();
    let _entity = world.spawn((A(3), B(5)));
    {
        let mut query = world.query::<All<(&mut A, &mut B)>>();
        // assert_eq!(query.archetypes_len(), 1);
        for (a, b) in query.iter_mut() {
            assert_eq!(a.0, 3);
            assert_eq!(b.0, 5);
        }
    }

    // let mut query_b = world.query_mut::<All<&mut A>>();
}

#[test]
fn query_multiple_mut() {
    let mut world = World::new();
    let _entity = world.spawn((A(3), B(5)));
    {
        let mut query = world.query_mut::<All<(&mut A, &mut B)>>();
        // assert_eq!(query.archetypes_len(), 1);
        for (a, b) in query.iter_mut() {
            assert_eq!(a.0, 3);
            assert_eq!(b.0, 5);
        }
    }

    // let mut query_b = world.query_mut::<All<&mut A>>();
}

#[test]
fn query_one() {
    let mut world = World::new();
    let _entity = world.spawn(A(3));
    {
        let query = world.query_mut::<One<&mut A>>();
        assert!(query.get().0 == 3)
    }
}
