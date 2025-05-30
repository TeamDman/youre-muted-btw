#[cfg(test)]
mod test {
    use ymb_ui_automation::gather_root;

    #[test]
    fn it_works() -> eyre::Result<()> {
        let root = gather_root()?;
        dbg!(&root);
        Ok(())
    }
}
