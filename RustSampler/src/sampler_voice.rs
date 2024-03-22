use std::clone;

use crate::ring_buffer;
use ring_buffer::RingBuffer;
use crate::adsr;
use adsr::ADSR;
#[derive(Clone)]
pub struct SamplerVoice{
    phase_offset: f32,
    phase_step: f32,
    pub midi_note: u8,
    pub base_midi: u8,
    num_channels: usize,
    pub adsr: ADSR,
    pub sus_is_velo: bool,
}

impl SamplerVoice{
    /// Deals with an audio file and either plays it back at a set rate
    /// based on a midi note or plays back an assigned file
    pub fn new(num_channesls_: usize, base_midi_: u8)->Self{
        let adsr_ = ADSR::new(44100.0, 0.2, 0.1,0.5,0.2);
        let mut voice = SamplerVoice{
            phase_offset: 0.0,
            phase_step: 1.0,
            midi_note: 0,
            base_midi: base_midi_,
            num_channels: num_channesls_,
            adsr: adsr_,
            sus_is_velo: false,

        };
        voice
    }
    ///Reads from the loaded sample file
    /// Uses the get_frac function in the ring_buffer, which returns the sample
    /// at a fractional index
    pub fn processWarp(&mut self, buffer: &mut RingBuffer<f32>, sr_scalar: f32)->f32{
        if self.adsr.is_active(){
            let sample = buffer.get_frac(self.phase_offset);
            self.phase_offset += self.phase_step * sr_scalar;
               
            if self.phase_offset >= buffer.capacity() as f32 {
                self.phase_step = 0.0;
                self.phase_offset = 0.0;
                //self.phase_offsets[self.channel_id] -= self.buffers[0].capacity() as f32;
            }
            sample * self.adsr.getNextSample()
        }else{
            self.phase_offset = 0.0;
            self.phase_step = 0.0;
            0.0
        }
    }
    /// Processes the voice when the sampler is in "Assign" mode.
    /// 
    /// Essentially, it just reads through the given buffer
    pub fn processAssign(&mut self, buffer: &mut RingBuffer<f32>, sr_scalar: f32)->f32{
        if self.adsr.is_active(){
            let sample = buffer.get_frac(self.phase_offset);
            self.phase_offset += 1.0 * sr_scalar;
            if self.phase_offset >= buffer.capacity() as f32 {
                self.phase_step = 0.0;
                self.phase_offset = 0.0;
            }
            sample*self.adsr.getNextSample()
        }else{
            self.phase_offset = 0.0;
            0.0
        }
    }
    ///Sets the midi note for the output
    /// 
    /// Is in reference to the base midi note
    pub fn set_note(&mut self, note: u8){
        self.midi_note = note;
        let offset = iclamp((note as i8 - self.base_midi as i8)as i32,-127,127);
        self.phase_step = 2.0_f32.powf(offset as f32 / 12.0);
    }
    /// Triggers attack on ADSR and starts playback of the audio file
    pub fn note_on(&mut self, note: u8, velocity: f32){
        if self.sus_is_velo {
            self.adsr.set_sustain(velocity);
        }
        self.phase_offset = 0.0;
        self.set_note(note);
        self.adsr.note_on();
    }
    /// Triggers release on ADSR
    pub fn note_off(&mut self){
        self.adsr.note_off()
    }
    /// Sets the attack, decay, sustain, and release for the ADSR (in seconds)
    pub fn set_adsr(&mut self, attack_:f32, decay_:f32, sustain_:f32, release_:f32){
        if !self.sus_is_velo{
            self.adsr.set_sustain(sustain_);
        }
        self.adsr.set_attack(attack_);
        self.adsr.set_decay(decay_);
        self.adsr.set_release(release_);
    }
    /// Returns whether or not the ADSR is active.
    /// 
    /// Useful for voice allocation
    pub fn is_active(&mut self)->bool{
        self.adsr.is_active()
    }
    /// Sets center midi note upon which sample warping is wrapped
    pub fn set_base_midi(&mut self, note: u8){
        self.base_midi = note;
    }

}


/// Clamps floats between a min and a max
fn fclamp(x: f32, min_val: f32, max_val: f32) -> f32 {
    if x < min_val {
        min_val
    } else if x > max_val {
        max_val
    } else {
        x
    }
}
/// Clamps ints between a min and a max
fn iclamp(x: i32, min_val: i32, max_val: i32) -> i32 {
    if x < min_val {
        min_val
    } else if x > max_val {
        max_val
    } else {
        x
    }
}