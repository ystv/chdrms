import { getCurrentUser } from '#/client';
import { Shell } from '#/components/shell';
import { createFileRoute, Outlet, redirect } from '@tanstack/react-router';

export const Route = createFileRoute('/(authenticated)')({
  component: RouteComponent,
  beforeLoad: async () => {
    const me = await getCurrentUser();

    if (!me.data) {
      throw redirect({
        to: '/login',
        search: {
          redirect: `${location.pathname}${location.search}${location.hash}`,
        },
      });
    }

    const postLoginRedirect = await cookieStore.get('rms_postlogin_redirect');
    if (postLoginRedirect?.value) {
      const fullURL = new URL(postLoginRedirect.value, location.origin);
      const redirectPath = `${fullURL.pathname}${fullURL.search}${fullURL.hash}`;
      console.log(`Post-login cookie detected, redirecting to ${redirectPath}`);
      await cookieStore.delete('rms_postlogin_redirect');
      throw redirect({
        href: redirectPath,
      });
    }
  },
});

function RouteComponent() {
  return (
    <Shell>
      <Outlet />
    </Shell>
  );
}
