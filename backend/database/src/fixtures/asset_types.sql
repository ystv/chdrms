INSERT INTO users(id, email, name)
VALUES ('736bcb69-ae67-4ec1-8868-cca4662aa3b1', 'test@example.com', 'Test User');

INSERT INTO manufacturers(id, name, created_by)
VALUES ('3d6fd755-8d90-4a86-881f-4870049bf5f9', 'Test Manufacturer', '736bcb69-ae67-4ec1-8868-cca4662aa3b1');

INSERT INTO asset_types(id, name, manufacturer, product_url, created_by)
VALUES ('f1c8508a-7c1d-436d-a867-0849dddf5f87', 'Test Asset Type', '3d6fd755-8d90-4a86-881f-4870049bf5f9', 'https://example.com', '736bcb69-ae67-4ec1-8868-cca4662aa3b1')
