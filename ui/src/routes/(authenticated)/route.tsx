import { Shell } from '#/components/shell';
import { createFileRoute, Outlet } from '@tanstack/react-router';

export const Route = createFileRoute('/(authenticated)')({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <Shell>
      <Outlet />
    </Shell>
  );
}
