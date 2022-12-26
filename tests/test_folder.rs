#[cfg(test)]

mod test_folder {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
   
    #[test]
    fn test_folder() {
        let mut f = encrypter::folder::FolderNode {
            path: "/home/use".into(),
            expanded: false,
            is_folder: true,
            subfolders: vec![],
        };

        encrypter::folder::expand(&mut f).unwrap();

        println!("{:?} \n", f.subfolders);
        println!("{} \n", f.subfolders[0].name())
    }
}
