pub mod axis;

use ndhistogram::{axis::{Axis, Uniform}, Histogram};
use axis::Cyclic;
use crate::io::hdf5::Hdf5Lor;
use crate::exports::{LOR, Point, Length};
use crate::exports::{Lengthf32, Ratiof32, Anglef32};
use geometry::uom::{mm_, mm, ns, ratio};
use std::f32::consts::PI;

/// Distinguish between true, scatter and random prompt signals
pub enum Prompt { True, Scatter, Random }

pub struct Scattergram {
    trues  : Box<dyn Lorogram>,
    scatters:Box<dyn Lorogram>,
}

impl Scattergram {

    pub fn new(make_empty_lorogram: &(dyn Fn() -> Box<dyn Lorogram>)) -> Self {
        let trues    = make_empty_lorogram();
        let scatters = make_empty_lorogram();
        Self { trues, scatters }
    }

    pub fn fill(&mut self, kind: Prompt, lor: &LOR) {
        match kind {
            Prompt::True    => self.trues.   fill(lor),
            Prompt::Scatter => self.scatters.fill(lor),
            Prompt::Random  => panic!("Not expecting any random events yet."),
        }
    } 

    /// Multiplicative contribution of scatters to trues, in nearby LORs.
    ///
    /// `(scatters + trues) / trues`
    pub fn value(&self, lor: &LOR) -> Ratiof32 {
        let trues = self.trues.value(lor) as f32;
        let scatters = self.scatters.value(lor) as f32;
        scatters / (trues + scatters)
    }

    pub fn triplet(&self, lor: &LOR) -> (Ratiof32, Ratiof32, Ratiof32) {
        let trues = self.trues.value(lor) as f32;
        let scatters = self.scatters.value(lor) as f32;
        (scatters / (trues + scatters), trues, scatters)
    }
}
// --------------------------------------------------------------------------------
pub struct MappedAxis<T,A>
where
    A: Axis,
{
    axis: A,
    map: Box<dyn Fn(&T) -> A::Coordinate>,
}

impl<T,A> Axis for MappedAxis<T,A>
where
    A: Axis,
{
    type Coordinate = T;

    type BinInterval = A::BinInterval;

    fn index(&self, coordinate: &Self::Coordinate) -> Option<usize> {
        self.axis.index(&(self.map)(coordinate))
    }

    fn num_bins(&self) -> usize {
        self.axis.num_bins()
    }

    fn bin(&self, index: usize) -> Option<Self::BinInterval> {
        self.axis.bin(index)
    }
}
// --------------------------------------------------------------------------------
pub type LorAxU = MappedAxis<LOR, Uniform<Lengthf32>>;
pub type LorAxC = MappedAxis<LOR, Cyclic <Lengthf32>>;

fn z_of_midpoint(LOR {p1, p2, ..}: &LOR) -> Lengthf32 { mm_(p1.z + p2.z) / 2.0 }

fn delta_z(LOR{p1, p2, ..}: &LOR) -> Lengthf32 { mm_(p1.z - p2.z).abs() }

fn distance_from_z_axis(LOR{ p1, p2, .. }: &LOR) -> Lengthf32 {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let x1 = p1.x;
    let y1 = p1.y;
    mm_((dx * y1 - dy * x1).abs() / (dx*dx + dy*dy).sqrt())
}

fn phi(LOR{ p1, p2, .. }: &LOR) -> Anglef32 {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    phi_of_x_y(mm_(dx), mm_(dy))
}

fn phi_of_x_y(x: Lengthf32, y: Lengthf32) -> Anglef32 { y.atan2(x) }

pub fn axis_z(nbins: usize, min: Length, max: Length) -> LorAxU {
    LorAxU {
        axis: Uniform::new(nbins, mm_(min), mm_(max)),
        map: Box::new(z_of_midpoint),
    }
}

pub fn axis_dz(nbins: usize, max: Length) -> LorAxU {
    LorAxU {
        axis: Uniform::new(nbins, 0.0, mm_(max)),
        map: Box::new(delta_z),
    }
}

pub fn axis_r(nbins: usize, max: Length) -> LorAxU {
    LorAxU {
        axis: Uniform::new(nbins, 0.0, mm_(max)),
        map: Box::new(distance_from_z_axis),
    }
}

pub fn axis_phi(nbins: usize) -> LorAxC {
    LorAxC {
        axis: Cyclic::new(nbins, 0.0, PI),
        map: Box::new(phi),
    }
}

#[cfg(test)]
mod test_mapped_axes {
    use super::*;
    use ndhistogram::ndhistogram;

    #[test]
    fn uniform() {
        let nbins = 10;
        let axis = axis_phi(nbins);
        assert_eq!(axis.num_bins(), nbins);
        let mut h = ndhistogram!(axis; usize);
        let x = mm(150.0);
        let y = mm(234.5);
        let (dummy1, dummy2, dummy3, dummy4) = (mm(111.1), mm(222.2), mm(333.3), mm(444.4));
        let (a, b) = (30.0, 40.0); // scaling factors
        Lorogram::fill         (&mut h, &mk_lor(((a*x, a*y, dummy1), (-a*x, -a*y, dummy2))));
        let n = Lorogram::value(&    h, &mk_lor(((b*x, b*y, dummy3), (-b*x, -b*y, dummy4))));
        assert_eq!(n, 1);
    }

    #[test]
    fn two_dimensions() {
        let nbins_z = 10;
        let nbins_dz = 10;
        let l = mm(1000.0);
        let max_dz = l;
        let mut h = ndhistogram!(
            axis_z (nbins_z , -l/2.0, l/2.0),
            axis_dz(nbins_dz, max_dz);
            usize
        );
        let (z, delta) = (mm(123.4), mm(543.2));
        // Irrelevant values
        let (i1, i2, i3, i4, i5, i6, i7, i8) = (mm(10.0), mm(20.0), mm(30.0), mm(40.0), mm(50.0), mm(60.0), mm(70.0), mm(80.0));

        let l1 = mk_lor(((i1, i2, z-delta), (i3, i4, z+delta)));
        let l2 = mk_lor(((i5, i6, z+delta), (i7, i8, z-delta)));
        Lorogram::fill         (&mut h, &l1);
        let n = Lorogram::value(&    h, &l2);

        assert_eq!(n, 1);
    }

}
// --------------------------------------------------------------------------------
pub trait Lorogram {
    fn fill (&mut self, lor: &LOR);
    fn value(&    self, lor: &LOR) -> usize;
}

impl<X> Lorogram for ndhistogram::Hist1D<X, usize>
where
    X: Axis<Coordinate = LOR>,
{
    fn fill (&mut self, lor: &LOR)          {  Histogram::fill (self, lor) }
    fn value(&    self, lor: &LOR) -> usize { *Histogram::value(self, lor).unwrap_or(&0) }
}

impl<X, Y> Lorogram for ndhistogram::Hist2D<X, Y, usize>
where
    X: Axis<Coordinate = LOR>,
    Y: Axis<Coordinate = LOR>,
{
    fn fill (&mut self, lor: &LOR)          {  Histogram::fill (self, &(*lor, *lor)) }
    fn value(&    self, lor: &LOR) -> usize { *Histogram::value(self, &(*lor, *lor)).unwrap_or(&0) }
}

impl<X, Y, Z> Lorogram for ndhistogram::Hist3D<X, Y, Z, usize>
where
    X: Axis<Coordinate = LOR>,
    Y: Axis<Coordinate = LOR>,
    Z: Axis<Coordinate = LOR>,
{
    fn fill (&mut self, lor: &LOR)          {  Histogram::fill (self, &(*lor, *lor, *lor)) }
    fn value(&    self, lor: &LOR) -> usize { *Histogram::value(self, &(*lor, *lor, *lor)).unwrap_or(&0) }
}

impl<X, Y, Z, T> Lorogram for ndhistogram::HistND<(X, Y, Z, T), usize>
where
    X: Axis<Coordinate = LOR>,
    Y: Axis<Coordinate = LOR>,
    Z: Axis<Coordinate = LOR>,
    T: Axis<Coordinate = LOR>,
{
    fn fill (&mut self, lor: &LOR)          {  Histogram::fill (self, &(*lor, *lor, *lor, *lor)) }
    fn value(&    self, lor: &LOR) -> usize { *Histogram::value(self, &(*lor, *lor, *lor, *lor)).unwrap_or(&0) }
}

pub fn fill_scattergram(make_empty_lorogram: &(dyn Fn() -> Box<dyn Lorogram>), lors: ndarray::Array1<Hdf5Lor>) ->  Scattergram {
    let mut sgram = Scattergram::new(make_empty_lorogram);
    for h5lor @Hdf5Lor { x1, x2, E1, E2, .. } in lors {
        if x1.is_nan() || x2.is_nan() { continue }
        let prompt = if E1.min(E2) < 511.0 { Prompt::Scatter } else { Prompt::True };
        sgram.fill(prompt, &LOR::from(h5lor));
    }
    sgram
}

pub fn mk_lor(((x1,y1,z1), (x2,y2,z2)): ((Length, Length, Length), (Length, Length, Length))) -> LOR {
    LOR { p1: Point::new(x1,y1,z1), p2: Point::new(x2,y2,z2), dt: ns(0.0), additive_correction: ratio(1.0) }
}
