use crate::geometry::{PhaseSample, SpiralGeometry};

/*
    Seriously I can't read this piece of code.
*/

pub fn fit(cur: &SpiralGeometry, error: &Vec<PhaseSample>, pivot_radius: f64) -> Option<SpiralGeometry> {

    let pivot_index = error
        .iter()
        .enumerate()
        .find(|(_, sample)| sample.radius_mm == pivot_radius)?.0;

    let (x_list, base_err): (Vec<f64>, Vec<f64>) = error.into_iter()
        .map(|s| 
            (cur.pos_at_radius(s.radius_mm), s.correction_rev)
        )
        .unzip();

    let base_rev: Vec<f64> = x_list.iter().map(|x| cur.polar_at_pos(*x).1).collect();

    let mut geometry = *cur;

    loop {
        let err: Vec<f64> = x_list.iter()
            .zip(base_rev.iter())
            .zip(base_err.iter())
            .map(|((x, old_n), abs_err)| {
                abs_err - (geometry.polar_at_pos(*x).1 - old_n)
            }).collect();
        println!("Remaining error: {:#?}", err);

        let r0 = geometry.r0_mm();
        let p = geometry.pitch_mm();
        let rev = geometry.revolutions();
        // Derivative n/N and n/p
        let (u, v): (Vec<f64>, Vec<f64>) = x_list.iter().map(|x| {
            let (r, n) = geometry.polar_at_pos(*x);
            let tn = (r0 + p * rev) * x / r;
            let tp = (rev * tn - n) / p;
            (tn, tp)
        })
        .unzip();

        let uk = u[pivot_index];
        let vk = v[pivot_index];
        let ek = err[pivot_index];

        let dnf = |p: f64| -> f64 {
            (ek - vk * p) / uk
        };
        
        let basis: Vec<f64> = u.iter().zip(v).map(|(ui, vi)| -> f64 {
            vi - (ui * vk / uk)
        }).collect();

        let resid_error: Vec<f64> = err.iter().zip(u).map(|(err, ui)| -> f64 {
            err - (ui * ek / uk)
        }).collect();
        
        let sum_basis: f64 = basis.iter().map(|x| x * x).sum();
        let sum_ve: f64 = basis.iter().zip(resid_error).map(|(v, e)| v * e).sum();

        let dp = sum_ve / sum_basis;
        let dn = dnf(dp);

        let new_geometry = SpiralGeometry::new(
            r0,
            geometry.pitch_um() + dp * 1000.0,
            rev + dn
        );

        println!("Delta-N = {dn:.6}, Delta-p = {:.9} um", dp * 1000.0);

        geometry = new_geometry;

        if dn.abs() < 1e-9 && dp.abs() < 1e-9 {
            return Some(geometry)
        }
        
    }
}