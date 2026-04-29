import { HeaderButton } from "./HeaderButton";
import type { NavigationProps } from "../types/navigation";

export type HeaderShellProps = NavigationProps & {
  sticky?: boolean;
};

export function HeaderShell(props: HeaderShellProps) {
  return <HeaderButton label={props.title} />;
}

export const HeaderNavigation = () => <HeaderShell title="Home" />;
