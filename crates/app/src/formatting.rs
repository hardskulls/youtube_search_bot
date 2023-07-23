
use std::io::Write;
use env_logger::fmt::Formatter;
use log::Record;


pub fn format_logs(buf: &mut Formatter, record: &Record) -> std::io::Result<()>
{
    let file = record.file().unwrap_or("unknown file");
    let line = record.line().unwrap_or(0);
    let time = chrono::Local::now().format("DATE ~ %Y/%m/%d || TIME ~ %H:%M:%S");
    let (level, args) = (record.level(), record.args());
    let separator = "===================================================================";
    
    writeln!
    (
        buf,
        "\
        \n{separator} \
        \n\nLOG : {level} \
        \n   ->   LOGGED AT ~ {file}:{line} \
        \n   ->   {time} \
        \n\n{args} \
        \n\n{separator}\n\n\n \
        "
    )
}


