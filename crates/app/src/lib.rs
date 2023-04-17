
pub mod formatting
{
    use std::io::Write;
    use env_logger::fmt::Formatter;
    use log::Record;

    pub fn format_logs(buf : &mut Formatter, record : &Record) -> std::io::Result<()>
    {

        let (file, line) = (record.file().unwrap_or("unknown file"), record.line().unwrap_or(0));
        let time = chrono::Local::now().format("Date %Y.%m.%d | Time %H:%M:%S");
        let (level, args) = (record.level(), record.args());

        writeln!(buf, " [:: LOG ::]:[ {} ] : [Location '{}:{}'] : [{}] :: ( {} ) ", level, file, line, time, args)
    }
}


