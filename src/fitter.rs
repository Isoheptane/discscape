use crate::geometry::{PhaseSample, SpiralGeometry};

fn partial_derivative_s(geometry: &SpiralGeometry, s: f64) -> (f64, f64) {
    let r0 = geometry.r0_mm();
    let p = geometry.pitch_mm();
    let rev = geometry.revolutions();
    let (r, n) = geometry.polar_at_pos(s);
    let dndn = (r0 + p * rev) * s / r;
    let dndp = (rev * dndn - n) / p;
    (dndn, dndp)
}

#[derive(Debug, thiserror::Error)]
pub enum FittingError {
    #[error("Failed to converge in {max_iter} iterations")]
    ConvergeFailed { max_iter: u64 },

    #[error("Singular matrix encountered during fitting (det is near zero)")]
    SingularMatrix,
}

/// Fitting a geometry's p and N to error
pub fn fit_rev_p(
    init: SpiralGeometry,
    error: &[PhaseSample],
    z_radius: f64
) -> Result<(SpiralGeometry, Vec<PhaseSample>), FittingError> {
    let count = error.len();

    let pos: Vec<f64> = error.iter()
        .map(|s| init.pos_at_radius(s.radius_mm))
        .collect();

    let init_rev: Vec<f64> = pos.iter()
        .map(|x| init.polar_at_pos(*x).1)
        .collect();

    let z_pos = init.pos_at_radius(z_radius);
    let z_init_rev = init.polar_at_pos(z_pos).1;

    let mut geometry = init;

    let mut residual = error.to_vec();

    const MAX_ITER: usize = 128;
    for _step_iter in 1..=MAX_ITER {
        let z_rev = geometry.polar_at_pos(z_pos).1;
        let z_correction = z_rev - z_init_rev;
        let (z_dndn, z_dndp) = partial_derivative_s(&geometry, z_pos);

        let mut s11 = 0.0;
        let mut s22 = 0.0;
        let mut s12 = 0.0;
        let mut pn = 0.0;
        let mut pp_um = 0.0;

        for i in 0..count {
            let i_correction = geometry.polar_at_pos(pos[i]).1 - init_rev[i];
            residual[i].rev = error[i].rev - (i_correction - z_correction);

            let (dndn, dndp) = partial_derivative_s(&geometry, pos[i]);
            let cn = dndn - z_dndn;

            // Pre-divide sensitivity , unit becomes N/um instead of N/mm
            // Minimalizes det-value
            let cp_um = (dndp - z_dndp) / 1000.0;

            s11 += cn * cn;
            s22 += cp_um * cp_um;
            s12 += cn * cp_um;
            pn += cn * residual[i].rev;
            pp_um += cp_um * residual[i].rev;
        }

        let det = s11 * s22 - (s12 * s12);

        if det < 1e-15 || det.is_nan() {
            return Err(FittingError::SingularMatrix);
        }
        
        let dn: f64 = (pn * s22 - pp_um * s12) / det;
        let dp_um: f64 = (pp_um * s11 - pn * s12) / det;

        println!("det={det}, dn={dn}, dp={dp_um}");

        // println!("{:#?}", residual);
        // println!("Delta-N: {:+}; Delta-p: {:+} nm", dn, dp * 1000000.0);

        geometry = SpiralGeometry::new(
            geometry.r0_mm(),
            geometry.pitch_um() + dp_um,
            geometry.revolutions() + dn
        );

        if dn.abs() < 1e-9 && dp_um.abs() < 1e-9 {
            // println!("Converged in {step_iter} iterations:");
            // println!("  Delta N: {:+.9}", geometry.revolutions() - init.revolutions());
            // println!("  Delta P: {:+.9} um", geometry.pitch_um() - init.pitch_um());
            // println!("  Residual Error:\n{:#?}", residual);
            return Ok((geometry, residual));
        }
    }

    Err(FittingError::ConvergeFailed { max_iter: MAX_ITER as u64 })
}


pub fn fit(init: &SpiralGeometry, error: &Vec<PhaseSample>, pivot_radius: f64) -> Option<SpiralGeometry> {

    let pivot_index = error
        .iter()
        .enumerate()
        .find(|(_, sample)| sample.radius_mm == pivot_radius)?.0;

    let (x_list, base_err): (Vec<f64>, Vec<f64>) = error.into_iter()
        .map(|s| 
            (init.pos_at_radius(s.radius_mm), s.rev)
        )
        .unzip();

    let base_rev: Vec<f64> = x_list.iter().map(|x| init.polar_at_pos(*x).1).collect();

    let mut geometry = *init;

    let mut step_count = 0;

    loop {
        let err: Vec<f64> = x_list.iter()
            .zip(base_rev.iter())
            .zip(base_err.iter())
            .map(|((x, old_n), abs_err)| {
                abs_err - (geometry.polar_at_pos(*x).1 - old_n)
            }).collect();
        // println!("Remaining error: {:#?}", err);

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
        let sum_ve: f64 = basis.iter().zip(resid_error.iter()).map(|(v, e)| v * e).sum();

        let dp = sum_ve / sum_basis;
        let dn = dnf(dp);

        let new_geometry = SpiralGeometry::new(
            r0,
            geometry.pitch_um() + dp * 1000.0,
            rev + dn
        );

        // println!("Delta-N = {dn:.6}, Delta-p = {:.9} um", dp * 1000.0);

        geometry = new_geometry;

        step_count += 1;
        

        if dn.abs() < 1e-9 && dp.abs() < 1e-12 {
            println!("Converged in {step_count} steps:");
            println!("  Delta N: {}", geometry.revolutions() - init.revolutions());
            println!("  Delta P: {} um", geometry.pitch_um() - init.pitch_um());
            println!("  Residual Error:\n{:#?}", resid_error);
            return Some(geometry);
        }
        
    }
}