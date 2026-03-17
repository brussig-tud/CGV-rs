
//////
//
// Imports
//

// Standard library
use std::path::PathBuf;

// Local imports
use cgv_util::uuid::Uuid;
use crate::compile::environment::*;



//////
//
// Tests
//

#[test]
fn test_add_module()
{
	let mut env = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "test");
	let path = "/test/module";
	let module = BytesModule::new(vec![1, 2, 3]);

	assert!(env.addModule(path, module.clone()).is_ok());
	assert_eq!(env.numModules(), 1);

	// Duplicate path should fail
	let res = env.addModule(path, module);
	assert!(res.is_err());
	if let Err(AddModuleError::DuplicateModulePaths(p)) = res {
		assert_eq!(p, PathBuf::from(path));
	} else {
		panic!("Expected DuplicateModulePaths error");
	}
}

#[test]
fn test_clone_with_new_uuid()
{
	let mut env1 = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "env1");
	let path = "/test/module";
	env1.addModule(path, BytesModule::new(vec![1, 2, 3])).unwrap();

	let env2uuid = Uuid::new_v4();
	let env2 = env1.cloneWithNewUuid(env2uuid, "env2");
	assert_eq!(env2.uuid(), env2uuid);
	assert_eq!(env2.label(), "env2");

	let moduleEntry = env2.modules().next().unwrap();
	assert_eq!(moduleEntry.path, PathBuf::from(path));
	assert_eq!(moduleEntry.sourceEnv, Some(env1.uuid())); // was native to env1, so it should now be sourced from env1
}

#[test]
fn test_merge_incompatible()
{
	let mut env1 = Environment::<BytesModule>::withUuidAndCompatHash(
		Uuid::new_v4(), "env1", 1
	);
	let env2 = Environment::<BytesModule>::withUuidAndCompatHash(
		Uuid::new_v4(), "env2", 2
	);

	let res = env1.mergeWith(&env2);
	assert!(matches!(res, Err(MergeError::Incompatible)));
}

#[test]
fn test_merge_identical_uuid()
{
	let uuid = Uuid::new_v4();
	let mut env1 = Environment::<BytesModule>::withUuid(uuid, "env1");
	let mut env2 = Environment::<BytesModule>::withUuid(uuid, "env2");
	env2.addModule("/test/module", BytesModule::new(vec![1, 2, 3])).unwrap();

	let res = env1.mergeWith(&env2);
	assert!(res.is_ok());
	assert_eq!(env1.numModules(), 0); // should have been a no-op, so no actual merging of modules should have happened
}

#[test]
fn test_merge_simple()
{
	let mut env1 = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "env1");
	let mut env2 = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "env2");

	env1.addModule("/a", BytesModule::new(vec![1])).unwrap();
	env2.addModule("/b", BytesModule::new(vec![2])).unwrap();

	env1.mergeWith(&env2).unwrap();
	assert_eq!(env1.numModules(), 2);

	let mod_a = env1.modules().find(
		|m| m.path == PathBuf::from("/a")
	).unwrap();
	assert_eq!(mod_a.sourceEnv, None); // still native

	let mod_b = env1.modules().find(
		|m| m.path == PathBuf::from("/b")
	).unwrap();
	assert_eq!(mod_b.sourceEnv, Some(env2.uuid())); // sourced from env2
}

#[test]
fn test_merge_duplicate_paths_different_source()
{
	let mut env1 = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "env1");
	let mut env2 = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "env2");

	env1.addModule("/common", BytesModule::new(vec![1])).unwrap();
	env2.addModule("/common", BytesModule::new(vec![2])).unwrap();

	let res = env1.mergeWith(&env2);
	assert!(matches!(res, Err(MergeError::DuplicateModulePaths(_))));
}

#[test]
fn test_merge_complex_scenario()
{
	// `a` and `c` should be merged, where `c` is the result of merging `a` and `b`.
	let mut a = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "a");
	let mut b = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "b");

	a.addModule("/a_mod", BytesModule::new(vec![1])).unwrap();
	b.addModule("/b_mod", BytesModule::new(vec![2])).unwrap();

	let mut c = a.merge(&b, Uuid::new_v4(), "c").unwrap();
	assert_eq!(c.numModules(), 2); // `c` now has "/a_mod" (source: `a`) and "/b_mod" (source: `b`)

	// Now merge `a` into `c`
	c.mergeWith(&a).unwrap();
	assert_eq!(c.numModules(), 2); // should still have 2, no error

	// Now merge `c` into `a`
	let mut a_copy = a.clone();
	a_copy.mergeWith(&c).unwrap();
	assert_eq!(a_copy.numModules(), 2); // should now have "/a_mod" and "/b_mod"

	let b_mod_in_a = a_copy.modules().find(|m| m.path == PathBuf::from("/b_mod")).unwrap();
	assert_eq!(b_mod_in_a.sourceEnv, Some(b.uuid()));
}

#[test]
fn test_merge_same_source()
{
	let mut base = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "base");
	base.addModule("/common", BytesModule::new(vec![0])).unwrap();

	let mut env1 = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "env1");
	let mut env2 = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "env2");

	env1.mergeWith(&base).unwrap();
	env2.mergeWith(&base).unwrap();

	// Both env1 and env2 have /common sourced from base
	env1.mergeWith(&env2).unwrap();

	assert_eq!(env1.numModules(), 1);
	let common = env1.modules().find(
		|m| m.path == PathBuf::from("/common")
	).unwrap();
	assert_eq!(common.sourceEnv, Some(base.uuid()));
}

#[test]
fn test_merge_partially_same_source()
{
	let mut base = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "base");
	base.addModule("/common", BytesModule::new(vec![0])).unwrap();

	let mut env1 = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "env1");
	env1.addModule("/mod1", BytesModule::new(vec![1])).unwrap();
	let mut env2 = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "env2");
	env2.addModule("/mod2", BytesModule::new(vec![1])).unwrap();

	env1.mergeWith(&base).unwrap();
	env2.mergeWith(&base).unwrap();

	// Both env1 and env2 have /common sourced from base
	env1.mergeWith(&env2).unwrap();

	// After final merge, env1 should now have /common from base, /mod1 from self (i.e. src==None), and /mod2 from env2
	assert_eq!(env1.numModules(), 3);
	let common = env1.modules().find(
		|m| m.path == PathBuf::from("/common")
	).unwrap();
	assert_eq!(common.sourceEnv, Some(base.uuid()));
	let mod1 = env1.modules().find(
		|m| m.path == PathBuf::from("/mod1")
	).unwrap();
	assert_eq!(mod1.sourceEnv, None); // <- None is internally sourced
	let mod2 = env1.modules().find(
		|m| m.path == PathBuf::from("/mod2")
	).unwrap();
	assert_eq!(mod2.sourceEnv, Some(env2.uuid()));
}

#[test]
fn test_merge_immutable_partially_same_source()
{
	let mut base = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "base");
	base.addModule("/common", BytesModule::new(vec![0])).unwrap();

	let mut env1 = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "env1");
	env1.addModule("/mod1", BytesModule::new(vec![1])).unwrap();
	let mut env2 = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "env2");
	env2.addModule("/mod2", BytesModule::new(vec![1])).unwrap();

	env1.mergeWith(&base).unwrap();
	env2.mergeWith(&base).unwrap();
	// Both env1 and env2 have /common sourced from base

	// Merge into new env3 without own modules should cause all modules be externalized to their original sources
	let env3 = env1.merge(&env2, Uuid::new_v4(), "env3").unwrap();

	// After final merge, env3 should have /common from base, /mod1 from env1, and /mod2 from env2
	assert_eq!(env3.numModules(), 3);
	let common = env3.modules().find(
		|m| m.path == PathBuf::from("/common")
	).unwrap();
	assert_eq!(common.sourceEnv, Some(base.uuid()));
	let mod1 = env3.modules().find(
		|m| m.path == PathBuf::from("/mod1")
	).unwrap();
	assert_eq!(mod1.sourceEnv, Some(env1.uuid()));
	let mod2 = env3.modules().find(
		|m| m.path == PathBuf::from("/mod2")
	).unwrap();
	assert_eq!(mod2.sourceEnv, Some(env2.uuid()));
}

#[test]
fn test_serialization()
{
	let mut env_orig = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "test");
	env_orig.addModule("/a", BytesModule::new(vec![1])).unwrap();

	let bytes = env_orig.serialize();
	let env = Environment::<BytesModule>::deserialize(&bytes).unwrap();

	assert_eq!(env.uuid(), env_orig.uuid());
	assert_eq!(env.label(), env_orig.label());
	assert_eq!(env.numModules(), env_orig.numModules());
}

#[test]
fn test_properties()
{
	let uuid = Uuid::new_v4();
	let label = "test_env";
	let hash = 12345u64;
	let env = Environment::<BytesModule>::withUuidAndCompatHash(uuid, label, hash);

	assert_eq!(env.uuid(), uuid);
	assert_eq!(env.label(), label);
	assert_eq!(env.compatHash(), hash);
}

#[test]
fn test_merge_immutable()
{
	let mut env1 = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "env1");
	let mut env2 = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "env2");

	env1.addModule("/a", BytesModule::new(vec![1])).unwrap();
	env2.addModule("/b", BytesModule::new(vec![2])).unwrap();

	let mergedUuid = Uuid::new_v4();
	let env3 = env1.merge(&env2, mergedUuid, "env3").unwrap();
	assert_eq!(env3.uuid(), mergedUuid);
	assert_eq!(env3.label(), "env3");
	assert_eq!(env3.numModules(), 2);

	// Check that "/a" is sourced from env1
	let a_mod = env3.modules().find(
		|m| m.path == PathBuf::from("/a")
	).unwrap();
	assert_eq!(a_mod.sourceEnv, Some(env1.uuid()));

	// Check that "/b" is sourced from env2
	let b_mod = env3.modules().find(
		|m| m.path == PathBuf::from("/b")
	).unwrap();
	assert_eq!(b_mod.sourceEnv, Some(env2.uuid()));

	// Check that env1 is unchanged
	assert_eq!(env1.numModules(), 1);
}

#[test]
fn test_merge_three_way_conflict()
{
	let mut a = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "a");
	let mut b = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "b");
	let mut c = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "c");

	a.addModule("/common", BytesModule::new(vec![1])).unwrap();
	b.addModule("/common", BytesModule::new(vec![2])).unwrap();
	// `a` and `b` both have "/common" native to them.

	c.mergeWith(&a).unwrap();
	// `c` now has "/common" from a

	let res = c.mergeWith(&b);
	// This should fail because `b` has "/common" and `c` already has "/common" from a.
	assert!(matches!(res, Err(MergeError::DuplicateModulePaths(_))));
}

#[test]
fn test_merge_native_conflict()
{
	let mut a = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "a");
	let mut c = Environment::<BytesModule>::withUuid(Uuid::new_v4(), "c");

	a.addModule("/common", BytesModule::new(vec![1])).unwrap();
	c.addModule("/common", BytesModule::new(vec![2])).unwrap();

	let res = c.mergeWith(&a);
	assert!(matches!(res, Err(MergeError::DuplicateModulePaths(_))));
}
