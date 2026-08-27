import { getContactDetails } from '#/client';
import { Title } from '@mantine/core';
import { createFileRoute, notFound } from '@tanstack/react-router';
import z from 'zod';

export const Route = createFileRoute('/a/$assetTag/')({
  params: {
    parse: (params) => ({
      assetTag: z.string().parse(params.assetTag),
    }),
  },
  loader: async ({ params: { assetTag } }) => {
    const contactDetails = await getContactDetails({
      query: { asset: assetTag },
    });

    if (!contactDetails.data) {
      throw notFound();
    }

    return { contactDetails };
  },
  component: RouteComponent,
});

function RouteComponent() {
  const { assetTag } = Route.useParams();
  const { contactDetails } = Route.useLoaderData();

  return (
    <div style={{ display: 'grid', placeItems: 'center', height: '100vh' }}>
      <div style={{ display: 'grid', placeItems: 'center' }}>
        <Title>{assetTag}</Title>
        This asset belongs to{' '}
        <Title order={3}>{contactDetails.data.name}</Title>
        Please contact them at one of the following:
        {contactDetails.data.links.map((link) => {
          return (
            <a
              style={{ textDecoration: 'underline' }}
              target="_blank"
              href={link.link}
              key={link.link}
            >
              {link.label ?? link.link}
            </a>
          );
        })}
      </div>
    </div>
  );
}
