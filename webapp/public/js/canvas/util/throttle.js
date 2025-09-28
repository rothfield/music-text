export function throttle(func, limit) {
  let inThrottle = false;
  let lastArgs = null;
  return function throttled(...args) {
    if (!inThrottle) {
      func.apply(this, args);
      inThrottle = true;
      setTimeout(() => {
        inThrottle = false;
        if (lastArgs) {
          const la = lastArgs;
          lastArgs = null;
          throttled.apply(this, la);
        }
      }, limit);
    } else {
      lastArgs = args;
    }
  };
}

