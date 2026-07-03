fn main() -> Result<(), Box<dyn std::error::Error>> {
    slint_build::compile("ui/app-window.slint")?;

    #[cfg(windows)]
    {
        winresource::WindowsResource::new()
            .set_icon("assets/icon.ico")
            .compile()?;
    }

    Ok(())
}
