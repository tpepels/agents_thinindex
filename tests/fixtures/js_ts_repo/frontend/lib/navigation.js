import React, { useMemo as useHeaderMemo } from "react";
import * as Metrics from "./metrics";

export class NavigationStore {
  mount(target) {
    Metrics.track(target);
  }

  handleClick = () => {
    return React.Fragment;
  };
}

export function createNavigationStore() {
  return new NavigationStore();
}

export const useNavigationStore = () => {
  return useHeaderMemo(() => createNavigationStore(), []);
};

export { NavigationStore as RenamedNavigationStore };
