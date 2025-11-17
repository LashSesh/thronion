//! ODE-Integration
//!
//! Numerische Integratoren für Differentialgleichungen

/// Runge-Kutta 4. Ordnung Integrator
///
/// Löst dx/dt = f(t, x) mit Anfangsbedingung x(t0) = x0
pub struct RungeKutta4<F>
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
{
    /// Derivative-Funktion f(t, x)
    pub derivative: F,
}

impl<F> RungeKutta4<F>
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
{
    /// Erstellt neuen RK4-Integrator
    pub fn new(derivative: F) -> Self {
        Self { derivative }
    }

    /// Einzelner Zeitschritt
    ///
    /// # Arguments
    /// * `t` - Aktuelle Zeit
    /// * `x` - Aktueller Zustand
    /// * `dt` - Zeitschrittweite
    pub fn step(&self, t: f64, x: &[f64], dt: f64) -> Vec<f64> {
        let n = x.len();

        // k1 = f(t, x)
        let k1 = (self.derivative)(t, x);

        // k2 = f(t + dt/2, x + dt*k1/2)
        let x_temp: Vec<f64> = (0..n).map(|i| x[i] + 0.5 * dt * k1[i]).collect();
        let k2 = (self.derivative)(t + 0.5 * dt, &x_temp);

        // k3 = f(t + dt/2, x + dt*k2/2)
        let x_temp: Vec<f64> = (0..n).map(|i| x[i] + 0.5 * dt * k2[i]).collect();
        let k3 = (self.derivative)(t + 0.5 * dt, &x_temp);

        // k4 = f(t + dt, x + dt*k3)
        let x_temp: Vec<f64> = (0..n).map(|i| x[i] + dt * k3[i]).collect();
        let k4 = (self.derivative)(t + dt, &x_temp);

        // x_new = x + (dt/6)*(k1 + 2*k2 + 2*k3 + k4)
        (0..n)
            .map(|i| x[i] + (dt / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]))
            .collect()
    }

    /// Integriert über Zeitintervall [t0, tf]
    ///
    /// Returns: Liste von (t, x) Punkten
    pub fn integrate(&self, t0: f64, x0: &[f64], tf: f64, dt: f64) -> Vec<(f64, Vec<f64>)> {
        let mut result = Vec::new();
        let mut t = t0;
        let mut x = x0.to_vec();

        result.push((t, x.clone()));

        while t < tf {
            let step_size = dt.min(tf - t);
            x = self.step(t, &x, step_size);
            t += step_size;
            result.push((t, x.clone()));
        }

        result
    }
}

/// Euler-Forward Integrator (einfacher, weniger genau)
pub struct EulerForward<F>
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
{
    /// Derivative function f(t, x) that returns dx/dt
    pub derivative: F,
}

impl<F> EulerForward<F>
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
{
    /// Creates a new Euler forward integrator
    pub fn new(derivative: F) -> Self {
        Self { derivative }
    }

    /// Performs a single Euler step
    pub fn step(&self, t: f64, x: &[f64], dt: f64) -> Vec<f64> {
        let dx = (self.derivative)(t, x);
        x.iter()
            .zip(dx.iter())
            .map(|(&xi, &dxi)| xi + dt * dxi)
            .collect()
    }

    /// Integrates from t0 to tf with step size dt
    pub fn integrate(&self, t0: f64, x0: &[f64], tf: f64, dt: f64) -> Vec<(f64, Vec<f64>)> {
        let mut result = Vec::new();
        let mut t = t0;
        let mut x = x0.to_vec();

        result.push((t, x.clone()));

        while t < tf {
            let step_size = dt.min(tf - t);
            x = self.step(t, &x, step_size);
            t += step_size;
            result.push((t, x.clone()));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_rk4_constant() {
        // dx/dt = 0 => x(t) = x0
        let integrator = RungeKutta4::new(|_, _| vec![0.0]);

        let x_new = integrator.step(0.0, &[1.0], 0.1);
        assert_abs_diff_eq!(x_new[0], 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_rk4_linear() {
        // dx/dt = a => x(t) = x0 + a*t
        let a = 2.0;
        let integrator = RungeKutta4::new(move |_, _| vec![a]);

        let trajectory = integrator.integrate(0.0, &[0.0], 1.0, 0.1);

        // x(1) sollte ca. 2.0 sein
        let (t_final, x_final) = trajectory.last().unwrap();
        assert_abs_diff_eq!(*t_final, 1.0, epsilon = 1e-10);
        assert_abs_diff_eq!(x_final[0], 2.0, epsilon = 1e-6);
    }

    #[test]
    fn test_rk4_exponential() {
        // dx/dt = x => x(t) = x0 * e^t
        let integrator = RungeKutta4::new(|_, x| vec![x[0]]);

        let trajectory = integrator.integrate(0.0, &[1.0], 1.0, 0.01);

        // x(1) sollte ca. e sein
        let (_, x_final) = trajectory.last().unwrap();
        assert_abs_diff_eq!(x_final[0], std::f64::consts::E, epsilon = 1e-4);
    }

    #[test]
    fn test_euler_forward() {
        // dx/dt = x => x(t) = x0 * e^t
        let integrator = EulerForward::new(|_, x| vec![x[0]]);

        let trajectory = integrator.integrate(0.0, &[1.0], 1.0, 0.01);

        // Euler ist weniger genau als RK4
        let (_, x_final) = trajectory.last().unwrap();
        assert!((x_final[0] - std::f64::consts::E).abs() < 0.1);
    }

    #[test]
    fn test_multidimensional() {
        // dx/dt = y, dy/dt = -x (Harmonic oscillator)
        let integrator = RungeKutta4::new(|_, x| vec![x[1], -x[0]]);

        let x0 = vec![1.0, 0.0]; // Initial: x=1, v=0
        let trajectory = integrator.integrate(0.0, &x0, 2.0 * std::f64::consts::PI, 0.01);

        // Nach einer Periode sollte x wieder bei 1 sein
        let (_, x_final) = trajectory.last().unwrap();
        assert_abs_diff_eq!(x_final[0], 1.0, epsilon = 0.01);
        assert_abs_diff_eq!(x_final[1], 0.0, epsilon = 0.01);
    }
}
