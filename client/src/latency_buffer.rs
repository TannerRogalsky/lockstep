#[derive(Clone, Debug)]
pub struct LatencyBuffer {
    buffer: Vec<(shared::FrameIndex, instant::Instant)>,
    timings: std::collections::VecDeque<std::time::Duration>,
    timeout: std::time::Duration,
    lost_packet_count: usize,
    recvd_packet_count: usize,
}

impl LatencyBuffer {
    pub fn with_timeout(timeout: std::time::Duration) -> Self {
        Self {
            buffer: Default::default(),
            timings: std::collections::VecDeque::with_capacity(100),
            timeout,
            lost_packet_count: 0,
            recvd_packet_count: 0,
        }
    }

    pub fn send(&mut self, index: shared::FrameIndex) {
        self.buffer.push((index, instant::Instant::now()))
    }

    pub fn recv(&mut self, frame: shared::FrameIndex) -> Option<std::time::Duration> {
        let len = self.buffer.len();
        self.buffer.retain({
            let timeout = self.timeout;
            move |(_index, time)| time.elapsed() < timeout
        });
        self.lost_packet_count += len - self.buffer.len();

        for (index, (other, instant)) in self.buffer.iter().enumerate() {
            if frame == *other {
                let d = instant.elapsed();
                if self.timings.capacity() == self.timings.len() {
                    self.timings.pop_front();
                }
                self.timings.push_back(d);
                self.buffer.swap_remove(index);
                self.recvd_packet_count += 1;
                return Some(d);
            }
        }
        None
    }

    pub fn average_latency(&self) -> std::time::Duration {
        if self.timings.is_empty() {
            std::time::Duration::default()
        } else {
            self.timings.iter().sum::<std::time::Duration>() / self.timings.len() as u32
        }
    }

    pub fn packet_loss(&self) -> f32 {
        if self.recvd_packet_count == 0 {
            0.
        } else {
            self.lost_packet_count as f32 / self.recvd_packet_count as f32
        }
    }
}
