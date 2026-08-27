import { getAssetByIdOrTag } from '#/client';
import { getAssetByIdOrTagOptions } from '#/client/@tanstack/react-query.gen';
import {
  Button,
  Card,
  Group,
  Stack,
  Text,
  Title,
  Tooltip,
} from '@mantine/core';
import { useClipboard } from '@mantine/hooks';
import { useQuery } from '@tanstack/react-query';
import { createFileRoute, notFound } from '@tanstack/react-router';
import z from 'zod';

export const Route = createFileRoute('/(authenticated)/assets/$assetID/')({
  params: {
    parse: (params) => ({
      assetID: z.string().parse(params.assetID),
    }),
  },
  loader: async ({ params: { assetID } }) => {
    const asset = await getAssetByIdOrTag({ path: { id: assetID } });

    if (!asset.data) {
      throw notFound();
    }

    return { asset };
  },
  component: RouteComponent,
});

function RouteComponent() {
  const { assetID } = Route.useParams();
  const { asset: initialAsset } = Route.useLoaderData();

  const asset = useQuery({
    ...getAssetByIdOrTagOptions({ path: { id: assetID } }),
    initialData: initialAsset.data,
    retry: false,
  });

  const clipboard = useClipboard({ timeout: 1000 });

  return (
    <Card>
      <Stack>
        <Group>
          <Tooltip label={clipboard.copied ? 'Copied!' : 'Copy'}>
            <Title
              onClick={() => clipboard.copy(asset.data.tag)}
              style={{ cursor: 'pointer' }}
            >
              {asset.data.tag}
            </Title>
          </Tooltip>
          <Tooltip label={clipboard.copied ? 'Copied!' : 'Copy'}>
            <Button
              ml={'auto'}
              c={'dimmed'}
              variant="transparent"
              onClick={() => clipboard.copy(asset.data.id)}
            >
              {asset.data.id}
            </Button>
          </Tooltip>
        </Group>
      </Stack>
    </Card>
  );
}
