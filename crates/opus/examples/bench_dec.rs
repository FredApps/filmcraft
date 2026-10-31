//! Development decoder benchmark: bench_dec <rate> <reps> <out.f32|-> <file.opus|file.pkt>.
use filmcraft_opus::{Packet, StreamDecoder};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn ogg_packets(data: &[u8]) -> Result<Vec<Vec<u8>>> {
    let mut packets = Vec::new();
    let mut cur = Vec::new();
    let mut pos = 0usize;
    while pos < data.len() {
        let header = data.get(pos..pos.saturating_add(27)).ok_or("truncated Ogg header")?;
        if header.get(..4) != Some(b"OggS") {
            return Err("invalid Ogg capture pattern".into());
        }
        let nseg = usize::from(*header.get(26).ok_or("missing segment count")?);
        let start = pos.checked_add(27).ok_or("Ogg offset overflow")?;
        let end = start.checked_add(nseg).ok_or("Ogg table overflow")?;
        let table = data.get(start..end).ok_or("truncated Ogg segment table")?;
        let mut body = end;
        for &len in table {
            let end = body.checked_add(usize::from(len)).ok_or("Ogg body overflow")?;
            cur.extend_from_slice(data.get(body..end).ok_or("truncated Ogg segment")?);
            body = end;
            if len < 255 {
                packets.push(std::mem::take(&mut cur));
            }
        }
        pos = body;
    }
    if !cur.is_empty() {
        return Err("unfinished Ogg packet".into());
    }
    Ok(packets)
}

fn packet_file(data: &[u8]) -> Result<(usize, Vec<Vec<u8>>)> {
    let header = data.get(..3).ok_or("truncated packet-file header")?;
    let channels = usize::from(*header.first().ok_or("missing channel count")?);
    let mut pos = 3usize;
    let mut packets = Vec::new();
    while pos < data.len() {
        let length = data.get(pos..pos.saturating_add(2)).ok_or("truncated packet length")?;
        let len = usize::from(u16::from_le_bytes([length[0], length[1]]));
        pos += 2;
        let end = pos.checked_add(len).ok_or("packet length overflow")?;
        packets.push(data.get(pos..end).ok_or("truncated packet payload")?.to_vec());
        pos = end;
    }
    Ok((channels, packets))
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    let arg = |i: usize| args.get(i).ok_or("usage: bench_dec <rate> <reps> <out.f32|-> <file.opus|file.pkt>");
    let rate: u32 = arg(1)?.to_str().ok_or("rate must be UTF-8")?.parse()?;
    let reps: usize = arg(2)?.to_str().ok_or("repetitions must be UTF-8")?.parse()?;
    if reps == 0 {
        return Err("repetitions must be positive".into());
    }
    let out_path = arg(3)?;
    let path = std::path::Path::new(arg(4)?);
    let data = std::fs::read(path)?;
    let (channels, packets) = if path.extension().is_some_and(|x| x == "opus") {
        let packets = ogg_packets(&data)?;
        let head = packets.first().ok_or("missing OpusHead")?;
        if head.get(..8) != Some(b"OpusHead") {
            return Err("missing OpusHead signature".into());
        }
        let channels = usize::from(*head.get(9).ok_or("truncated OpusHead")?);
        (channels, packets.get(2..).ok_or("missing OpusTags")?.to_vec())
    } else {
        packet_file(&data)?
    };
    let mut modes = [0usize; 3];
    for packet in &packets {
        if let Ok(pk) = Packet::parse(packet)
            && let Some(count) = modes.get_mut(pk.toc.mode as usize)
        {
            *count += 1;
        }
    }
    let t0 = std::time::Instant::now();
    let mut out = Vec::new();
    for _ in 0..reps {
        out.clear();
        let mut decoder = StreamDecoder::new(rate, channels)?;
        for packet in &packets {
            decoder.decode_interleaved(Some(packet), &mut out)?;
        }
    }
    if out.is_empty() {
        return Err("input contains no decoded audio samples".into());
    }
    let dt = (t0.elapsed().as_secs_f64() / reps as f64).max(f64::MIN_POSITIVE);
    let secs = out.len() as f64 / channels as f64 / rate as f64;
    eprintln!("{}: ch={channels} modes={modes:?} {dt:.4}s per pass for {secs:.1}s audio ({:.0}x realtime)", path.display(), secs / dt);
    if out_path != "-" {
        let bytes: Vec<u8> = out.iter().flat_map(|v| v.to_le_bytes()).collect();
        std::fs::write(out_path, bytes)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncated_inputs_are_errors() {
        assert!(ogg_packets(b"OggS").is_err());
        let mut page = vec![0; 27];
        page[..4].copy_from_slice(b"OggS");
        page[26] = 1;
        assert!(ogg_packets(&page).is_err());
        page.push(12);
        assert!(ogg_packets(&page).is_err());
        for bytes in [&[][..], &[2], &[2, 0, 0, 1], &[2, 0, 0, 4, 0, 1]] {
            assert!(packet_file(bytes).is_err());
        }
    }
}
