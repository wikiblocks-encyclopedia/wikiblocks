use crate::{mock::*, pallet};

use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;

use sp_core::Pair;
use wikiblocks_primitives::{insecure_pair_from_name, ArticleVersion, Body, OpCode, Script, Title};

#[test]
fn add_article() {
  new_test_ext().execute_with(|| {
    let user = insecure_pair_from_name("user").public();
    let title = Title::new("example title".as_bytes().to_vec()).unwrap();
    let body = Body::new("this is an example article".as_bytes().to_vec()).unwrap();

    // send the script
    let script =
      Script::new(vec![OpCode::Title(title.clone()), OpCode::Add(body.clone())]).unwrap();
    assert_ok!(Articles::add_article(RawOrigin::Signed(user).into(), script.clone()));

    // check that titles have 1 item that is correct
    let titles = Articles::titles();
    assert_eq!(titles, vec![title.clone()]);

    // check that we have a version 0 for it
    let versions = Articles::versions(&title).unwrap();
    assert_eq!(versions, ArticleVersion(0));

    // check that we have a body for the version
    let in_chain_script = Articles::articles((&title, ArticleVersion(0))).unwrap();
    assert_eq!(in_chain_script, Script::new(vec![OpCode::Add(body)]).unwrap());

    // check the author is right
    let author = Articles::authors((&title, ArticleVersion(0))).unwrap();
    assert_eq!(author, user);
  })
}

#[test]
fn add_version() {
  new_test_ext().execute_with(|| {
    let user = insecure_pair_from_name("user").public();
    let title = Title::new("example title".as_bytes().to_vec()).unwrap();
    let body = Body::new("this is an example article".as_bytes().to_vec()).unwrap();

    // add a title first
    let script =
      Script::new(vec![OpCode::Title(title.clone()), OpCode::Add(body.clone())]).unwrap();
    assert_ok!(Articles::add_article(RawOrigin::Signed(user).into(), script));

    // add a new version for it
    let body2 =
      Body::new("\n this is a second line added the first version".as_bytes().to_vec()).unwrap();
    let script = Script::new(vec![
      OpCode::Reference(ArticleVersion(0)),
      OpCode::Cp(body.data().len().try_into().unwrap()), // copy all data from the ref version.
      OpCode::Add(body2),                                // continue by adding the body2 data
    ])
    .unwrap();
    assert_ok!(Articles::add_version(
      RawOrigin::Signed(user).into(),
      title.clone(),
      script.clone()
    ));

    // check that titles have 1 item that is correct
    let titles = Articles::titles();
    assert_eq!(titles, vec![title.clone()]);

    // check that we have 2 version of it
    let versions = Articles::versions(&title).unwrap();
    assert_eq!(versions, ArticleVersion(1)); // 0, 1

    // check that we have a body for the version
    let in_chain_script = Articles::articles((&title, ArticleVersion(1))).unwrap();
    assert_eq!(in_chain_script, script);
  })
}

#[test]
fn add_article_invalid_script() {
  new_test_ext().execute_with(|| {
    let user = insecure_pair_from_name("user").public();
    let title = Title::new("example title".as_bytes().to_vec()).unwrap();
    let body = Body::new("this is an example article".as_bytes().to_vec()).unwrap();

    // title should be first Opcode
    let opcodes = vec![OpCode::Add(body.clone()), OpCode::Title(title.clone())];
    let script = Script::new(opcodes).unwrap();
    assert_noop!(
      Articles::add_article(RawOrigin::Signed(user).into(), script),
      pallet::Error::<Test>::InvalidScript
    );

    // title opcode should be followed by add opcode
    let opcodes = vec![OpCode::Title(title.clone()), OpCode::Title(title.clone())];
    let script = Script::new(opcodes).unwrap();
    assert_noop!(
      Articles::add_article(RawOrigin::Signed(user).into(), script),
      pallet::Error::<Test>::InvalidScript
    );

    // can't have more than 2 opcode
    let opcodes =
      vec![OpCode::Title(title.clone()), OpCode::Add(body.clone()), OpCode::Add(body.clone())];
    let script = Script::new(opcodes).unwrap();
    assert_noop!(
      Articles::add_article(RawOrigin::Signed(user).into(), script),
      pallet::Error::<Test>::InvalidScript
    );

    // can't have empty Script
    let opcodes = vec![];
    let script = Script::new(opcodes).unwrap();
    assert_noop!(
      Articles::add_article(RawOrigin::Signed(user).into(), script),
      pallet::Error::<Test>::InvalidScript
    );

    // can't have empty title
    let opcodes = vec![OpCode::Title(Title::new(vec![]).unwrap()), OpCode::Add(body.clone())];
    let script = Script::new(opcodes).unwrap();
    assert_noop!(
      Articles::add_article(RawOrigin::Signed(user).into(), script),
      pallet::Error::<Test>::InvalidScript
    );

    // can't have empty body
    let opcodes = vec![OpCode::Title(title.clone()), OpCode::Add(Body::new(vec![]).unwrap())];
    let script = Script::new(opcodes).unwrap();
    assert_noop!(
      Articles::add_article(RawOrigin::Signed(user).into(), script),
      pallet::Error::<Test>::InvalidScript
    );

    // valid script
    let opcodes = vec![OpCode::Title(title.clone()), OpCode::Add(body.clone())];
    let script = Script::new(opcodes).unwrap();
    assert_ok!(Articles::add_article(RawOrigin::Signed(user).into(), script));

    // can't add article with the same title
    let body = Body::new("this is second body".as_bytes().to_vec()).unwrap();
    let script = Script::new(vec![OpCode::Title(title.clone()), OpCode::Add(body)]).unwrap();
    assert_noop!(
      Articles::add_article(RawOrigin::Signed(user).into(), script),
      pallet::Error::<Test>::TitleAlreadyExist
    );
  })
}

#[test]
fn add_version_invalid_script() {
  new_test_ext().execute_with(|| {
    let user = insecure_pair_from_name("user").public();
    let title = Title::new("example title".as_bytes().to_vec()).unwrap();
    let body = Body::new("this is an example article".as_bytes().to_vec()).unwrap();

    // valid an article first
    let opcodes = vec![OpCode::Title(title.clone()), OpCode::Add(body.clone())];
    let script = Script::new(opcodes).unwrap();
    assert_ok!(Articles::add_article(RawOrigin::Signed(user).into(), script));

    // can't have empty Script
    let script = Script::new(vec![]).unwrap();
    assert_noop!(
      Articles::add_version(RawOrigin::Signed(user).into(), title.clone(), script),
      pallet::Error::<Test>::InvalidScript
    );

    // title must exist
    let add_opcode = OpCode::Add(Body::new("valid body".as_bytes().to_vec()).unwrap());
    let script = Script::new(vec![add_opcode.clone()]).unwrap();
    assert_noop!(
      Articles::add_version(
        RawOrigin::Signed(user).into(),
        Title::new("invalid title".as_bytes().to_vec()).unwrap(),
        script
      ),
      pallet::Error::<Test>::InvalidTitle
    );

    // can't have a title opcode
    let script = Script::new(vec![OpCode::Title(title.clone()), add_opcode.clone()]).unwrap();
    assert_noop!(
      Articles::add_version(RawOrigin::Signed(user).into(), title.clone(), script),
      pallet::Error::<Test>::InvalidScript
    );

    // can't have a reference version that is invalid
    let script =
      Script::new(vec![OpCode::Reference(ArticleVersion(2)), add_opcode.clone()]).unwrap();
    assert_noop!(
      Articles::add_version(RawOrigin::Signed(user).into(), title.clone(), script),
      pallet::Error::<Test>::InvalidReference
    );

    // can't have an opcode that requires a reference without the reference opcode
    let script = Script::new(vec![OpCode::Cp(20), OpCode::End]).unwrap();
    assert_noop!(
      Articles::add_version(RawOrigin::Signed(user).into(), title.clone(), script),
      pallet::Error::<Test>::InvalidScript
    );
  })
}
