//! Reimplementation of `bevy_time::Time`. Use this instead of importing `bevy_time`.

use std::time::Duration;

use bevy::ecs::prelude::*;

use crate::ffi;

/// A clock that tracks how much it has advanced (and how much real time has elapsed) since
/// its previous update and since its creation.
#[derive(Resource, Debug, Clone)]
pub struct Time {
    startup: Duration,
    first_update: Option<Duration>,
    last_update: Option<Duration>,
    // pausing
    paused: bool,
    // scaling
    relative_speed: f64, // using `f64` instead of `f32` to minimize drift from rounding errors
    delta: Duration,
    delta_seconds: f32,
    delta_seconds_f64: f64,
    elapsed: Duration,
    elapsed_seconds: f32,
    elapsed_seconds_f64: f64,
    raw_delta: Duration,
    raw_delta_seconds: f32,
    raw_delta_seconds_f64: f64,
    raw_elapsed: Duration,
    raw_elapsed_seconds: f32,
    raw_elapsed_seconds_f64: f64,
    // wrapping
    wrap_period: Duration,
    elapsed_wrapped: Duration,
    elapsed_seconds_wrapped: f32,
    elapsed_seconds_wrapped_f64: f64,
    raw_elapsed_wrapped: Duration,
    raw_elapsed_seconds_wrapped: f32,
    raw_elapsed_seconds_wrapped_f64: f64,
}

impl Time {
    /// Constructs a new `Time` instance with a specific startup `Instant`.
    pub fn new() -> Self {
        Self {
            startup: unsafe { Duration::from_nanos(ffi::get_time_since_startup()) },
            first_update: None,
            last_update: None,
            paused: false,
            relative_speed: 1.0,
            delta: Duration::ZERO,
            delta_seconds: 0.0,
            delta_seconds_f64: 0.0,
            elapsed: Duration::ZERO,
            elapsed_seconds: 0.0,
            elapsed_seconds_f64: 0.0,
            raw_delta: Duration::ZERO,
            raw_delta_seconds: 0.0,
            raw_delta_seconds_f64: 0.0,
            raw_elapsed: Duration::ZERO,
            raw_elapsed_seconds: 0.0,
            raw_elapsed_seconds_f64: 0.0,
            wrap_period: Duration::from_secs(3600), // 1 hour
            elapsed_wrapped: Duration::ZERO,
            elapsed_seconds_wrapped: 0.0,
            elapsed_seconds_wrapped_f64: 0.0,
            raw_elapsed_wrapped: Duration::ZERO,
            raw_elapsed_seconds_wrapped: 0.0,
            raw_elapsed_seconds_wrapped_f64: 0.0,
        }
    }

    /// Updates the internal time measurements.
    ///
    /// Calling this method as part of your app will most likely result in inaccurate timekeeping,
    /// as the `Time` resource is ordinarily managed by bevy_wasm.
    pub fn update(&mut self) {
        let now = unsafe { ffi::get_time_since_startup() };
        self.update_with_instant(Duration::from_nanos(now));
    }

    /// Updates time with a specified [`Duration`].
    ///
    /// This method is provided for use in tests. Calling this method as part of your app will most
    /// likely result in inaccurate timekeeping, as the `Time` resource is ordinarily managed by bevy_wasm.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy_time::prelude::*;
    /// # use bevy_ecs::prelude::*;
    /// # use bevy_utils::Duration;
    /// # fn main () {
    /// #     test_health_system();
    /// # }
    /// #[derive(Resource)]
    /// struct Health {
    ///     // Health value between 0.0 and 1.0
    ///     health_value: f32,
    /// }
    ///
    /// fn health_system(time: Res<Time>, mut health: ResMut<Health>) {
    ///     // Increase health value by 0.1 per second, independent of frame rate,
    ///     // but not beyond 1.0
    ///     health.health_value = (health.health_value + 0.1 * time.delta_seconds()).min(1.0);
    /// }
    ///
    /// // Mock time in tests
    /// fn test_health_system() {
    ///     let mut world = World::default();
    ///     let mut time = Time::default();
    ///     time.update();
    ///     world.insert_resource(time);
    ///     world.insert_resource(Health { health_value: 0.2 });
    ///
    ///     let mut update_stage = SystemStage::parallel();
    ///     update_stage.add_system(health_system);
    ///
    ///     // Simulate that 30 ms have passed
    ///     let mut time = world.resource_mut::<Time>();
    ///     let last_update = time.last_update().unwrap(); // Don't unwrap in a mod
    ///     time.update_with_instant(last_update + Duration::from_millis(30));
    ///
    ///     // Run system
    ///     update_stage.run(&mut world);
    ///
    ///     // Check that 0.003 has been added to the health value
    ///     let expected_health_value = 0.2 + 0.1 * 0.03;
    ///     let actual_health_value = world.resource::<Health>().health_value;
    ///     assert_eq!(expected_health_value, actual_health_value);
    /// }
    /// ```
    pub fn update_with_instant(&mut self, instant: Duration) {
        let raw_delta = instant - self.last_update.unwrap_or(instant);
        let delta = if self.paused {
            Duration::ZERO
        } else if self.relative_speed != 1.0 {
            raw_delta.mul_f64(self.relative_speed)
        } else {
            // avoid rounding when at normal speed
            raw_delta
        };

        if self.last_update.is_some() {
            self.delta = delta;
            self.delta_seconds = self.delta.as_secs_f32();
            self.delta_seconds_f64 = self.delta.as_secs_f64();
            self.raw_delta = raw_delta;
            self.raw_delta_seconds = self.raw_delta.as_secs_f32();
            self.raw_delta_seconds_f64 = self.raw_delta.as_secs_f64();
        } else {
            self.first_update = Some(instant);
        }

        self.elapsed += delta;
        self.elapsed_seconds = self.elapsed.as_secs_f32();
        self.elapsed_seconds_f64 = self.elapsed.as_secs_f64();
        self.raw_elapsed += raw_delta;
        self.raw_elapsed_seconds = self.raw_elapsed.as_secs_f32();
        self.raw_elapsed_seconds_f64 = self.raw_elapsed.as_secs_f64();

        self.elapsed_wrapped = duration_div_rem(self.elapsed, self.wrap_period).1;
        self.elapsed_seconds_wrapped = self.elapsed_wrapped.as_secs_f32();
        self.elapsed_seconds_wrapped_f64 = self.elapsed_wrapped.as_secs_f64();
        self.raw_elapsed_wrapped = duration_div_rem(self.raw_elapsed, self.wrap_period).1;
        self.raw_elapsed_seconds_wrapped = self.raw_elapsed_wrapped.as_secs_f32();
        self.raw_elapsed_seconds_wrapped_f64 = self.raw_elapsed_wrapped.as_secs_f64();

        self.last_update = Some(instant);
    }

    /// Returns the [`Duration`] the clock was created.
    ///
    /// This usually represents when the app was started.
    #[inline]
    pub fn startup(&self) -> Duration {
        self.startup
    }

    /// Returns the [`Duration`] when [`update`](#method.update) was first called, if it exists.
    ///
    /// This usually represents when the first app update started.
    #[inline]
    pub fn first_update(&self) -> Option<Duration> {
        self.first_update
    }

    /// Returns the [`Duration`] when [`update`](#method.update) was last called, if it exists.
    ///
    /// This usually represents when the current app update started.
    #[inline]
    pub fn last_update(&self) -> Option<Duration> {
        self.last_update
    }

    /// Returns how much time has advanced since the last [`update`](#method.update), as a [`Duration`].
    #[inline]
    pub fn delta(&self) -> Duration {
        self.delta
    }

    /// Returns how much time has advanced since the last [`update`](#method.update), as [`f32`] seconds.
    #[inline]
    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds
    }

    /// Returns how much time has advanced since the last [`update`](#method.update), as [`f64`] seconds.
    #[inline]
    pub fn delta_seconds_f64(&self) -> f64 {
        self.delta_seconds_f64
    }

    /// Returns how much time has advanced since [`startup`](#method.startup), as [`Duration`].
    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Returns how much time has advanced since [`startup`](#method.startup), as [`f32`] seconds.
    ///
    /// **Note:** This is a monotonically increasing value. It's precision will degrade over time.
    /// If you need an `f32` but that precision loss is unacceptable,
    /// use [`elapsed_seconds_wrapped`](#method.elapsed_seconds_wrapped).
    #[inline]
    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed_seconds
    }

    /// Returns how much time has advanced since [`startup`](#method.startup), as [`f64`] seconds.
    #[inline]
    pub fn elapsed_seconds_f64(&self) -> f64 {
        self.elapsed_seconds_f64
    }

    /// Returns how much time has advanced since [`startup`](#method.startup) modulo
    /// the [`wrap_period`](#method.wrap_period), as [`Duration`].
    #[inline]
    pub fn elapsed_wrapped(&self) -> Duration {
        self.elapsed_wrapped
    }

    /// Returns how much time has advanced since [`startup`](#method.startup) modulo
    /// the [`wrap_period`](#method.wrap_period), as [`f32`] seconds.
    ///
    /// This method is intended for applications (e.g. shaders) that require an [`f32`] value but
    /// suffer from the gradual precision loss of [`elapsed_seconds`](#method.elapsed_seconds).
    #[inline]
    pub fn elapsed_seconds_wrapped(&self) -> f32 {
        self.elapsed_seconds_wrapped
    }

    /// Returns how much time has advanced since [`startup`](#method.startup) modulo
    /// the [`wrap_period`](#method.wrap_period), as [`f64`] seconds.
    #[inline]
    pub fn elapsed_seconds_wrapped_f64(&self) -> f64 {
        self.elapsed_seconds_wrapped_f64
    }

    /// Returns how much real time has elapsed since the last [`update`](#method.update), as a [`Duration`].
    #[inline]
    pub fn raw_delta(&self) -> Duration {
        self.raw_delta
    }

    /// Returns how much real time has elapsed since the last [`update`](#method.update), as [`f32`] seconds.
    #[inline]
    pub fn raw_delta_seconds(&self) -> f32 {
        self.raw_delta_seconds
    }

    /// Returns how much real time has elapsed since the last [`update`](#method.update), as [`f64`] seconds.
    #[inline]
    pub fn raw_delta_seconds_f64(&self) -> f64 {
        self.raw_delta_seconds_f64
    }

    /// Returns how much real time has elapsed since [`startup`](#method.startup), as [`Duration`].
    #[inline]
    pub fn raw_elapsed(&self) -> Duration {
        self.raw_elapsed
    }

    /// Returns how much real time has elapsed since [`startup`](#method.startup), as [`f32`] seconds.
    ///
    /// **Note:** This is a monotonically increasing value. It's precision will degrade over time.
    /// If you need an `f32` but that precision loss is unacceptable,
    /// use [`raw_elapsed_seconds_wrapped`](#method.raw_elapsed_seconds_wrapped).
    #[inline]
    pub fn raw_elapsed_seconds(&self) -> f32 {
        self.raw_elapsed_seconds
    }

    /// Returns how much real time has elapsed since [`startup`](#method.startup), as [`f64`] seconds.
    #[inline]
    pub fn raw_elapsed_seconds_f64(&self) -> f64 {
        self.raw_elapsed_seconds_f64
    }

    /// Returns how much real time has elapsed since [`startup`](#method.startup) modulo
    /// the [`wrap_period`](#method.wrap_period), as [`Duration`].
    #[inline]
    pub fn raw_elapsed_wrapped(&self) -> Duration {
        self.raw_elapsed_wrapped
    }

    /// Returns how much real time has elapsed since [`startup`](#method.startup) modulo
    /// the [`wrap_period`](#method.wrap_period), as [`f32`] seconds.
    ///
    /// This method is intended for applications (e.g. shaders) that require an [`f32`] value but
    /// suffer from the gradual precision loss of [`raw_elapsed_seconds`](#method.raw_elapsed_seconds).
    #[inline]
    pub fn raw_elapsed_seconds_wrapped(&self) -> f32 {
        self.raw_elapsed_seconds_wrapped
    }

    /// Returns how much real time has elapsed since [`startup`](#method.startup) modulo
    /// the [`wrap_period`](#method.wrap_period), as [`f64`] seconds.
    #[inline]
    pub fn raw_elapsed_seconds_wrapped_f64(&self) -> f64 {
        self.raw_elapsed_seconds_wrapped_f64
    }

    /// Returns the modulus used to calculate [`elapsed_wrapped`](#method.elapsed_wrapped) and
    /// [`raw_elapsed_wrapped`](#method.raw_elapsed_wrapped).
    ///
    /// **Note:** The default modulus is one hour.
    #[inline]
    pub fn wrap_period(&self) -> Duration {
        self.wrap_period
    }

    /// Sets the modulus used to calculate [`elapsed_wrapped`](#method.elapsed_wrapped) and
    /// [`raw_elapsed_wrapped`](#method.raw_elapsed_wrapped).
    ///
    /// **Note:** This will not take effect until the next update.
    ///
    /// # Panics
    ///
    /// Panics if `wrap_period` is a zero-length duration.
    #[inline]
    pub fn set_wrap_period(&mut self, wrap_period: Duration) {
        assert!(!wrap_period.is_zero(), "division by zero");
        self.wrap_period = wrap_period;
    }

    /// Returns the speed the clock advances relative to your system clock, as [`f32`].
    /// This is known as "time scaling" or "time dilation" in other engines.
    ///
    /// **Note:** This function will return zero when time is paused.
    #[inline]
    pub fn relative_speed(&self) -> f32 {
        self.relative_speed_f64() as f32
    }

    /// Returns the speed the clock advances relative to your system clock, as [`f64`].
    /// This is known as "time scaling" or "time dilation" in other engines.
    ///
    /// **Note:** This function will return zero when time is paused.
    #[inline]
    pub fn relative_speed_f64(&self) -> f64 {
        if self.paused {
            0.0
        } else {
            self.relative_speed
        }
    }

    /// Sets the speed the clock advances relative to your system clock, given as an [`f32`].
    ///
    /// For example, setting this to `2.0` will make the clock advance twice as fast as your system clock.
    ///
    /// **Note:** This does not affect the `raw_*` measurements.
    ///
    /// # Panics
    ///
    /// Panics if `ratio` is negative or not finite.
    #[inline]
    pub fn set_relative_speed(&mut self, ratio: f32) {
        self.set_relative_speed_f64(ratio as f64);
    }

    /// Sets the speed the clock advances relative to your system clock, given as an [`f64`].
    ///
    /// For example, setting this to `2.0` will make the clock advance twice as fast as your system clock.
    ///
    /// **Note:** This does not affect the `raw_*` measurements.
    ///
    /// # Panics
    ///
    /// Panics if `ratio` is negative or not finite.
    #[inline]
    pub fn set_relative_speed_f64(&mut self, ratio: f64) {
        assert!(ratio.is_finite(), "tried to go infinitely fast");
        assert!(ratio.is_sign_positive(), "tried to go back in time");
        self.relative_speed = ratio;
    }

    /// Stops the clock, preventing it from advancing until resumed.
    ///
    /// **Note:** This does affect the `raw_*` measurements.
    #[inline]
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes the clock if paused.
    #[inline]
    pub fn unpause(&mut self) {
        self.paused = false;
    }

    /// Returns `true` if the clock is currently paused.
    #[inline]
    pub fn is_paused(&self) -> bool {
        self.paused
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

fn duration_div_rem(dividend: Duration, divisor: Duration) -> (u32, Duration) {
    // `Duration` does not have a built-in modulo operation
    let quotient = (dividend.as_nanos() / divisor.as_nanos()) as u32;
    let remainder = dividend - (quotient * divisor);
    (quotient, remainder)
}
