import pytest

from pydantic import BaseModel
from ordantic import ExampleModel, ExampleModel2

class ExampleModel3(BaseModel):
    model1: ExampleModel
    model2: ExampleModel2

@pytest.mark.skip
def test_hash():
    model1 = ExampleModel(name="foo", number=3)
    model2 = ExampleModel(name="foo", number=3)
    assert hash(model1) == hash(model2)

def test_dict():
    model = ExampleModel(name="foo", number=3)
    assert model.dict() == {"name": "foo", "number": 3}

def test_json():
    model = ExampleModel(name="foo", number=3)
    assert model.json() == '{"name":"foo","number":3}'

def test_invalid_json():
    model = ExampleModel(name="foo", number=11)
    assert model.json() == '{"name":"foo","number":11}'

def test_parse_raw():
    model = ExampleModel(name="foo", number=3)
    assert model == ExampleModel.parse_raw('{"name":"foo","number":3}')

def test_invalid_parse_raw():
    model = ExampleModel(name="foo", number=11)
    assert model == ExampleModel.parse_raw('{"name":"foo","number":11}')

def test_schema():
    schema = ExampleModel.schema()
    assert schema == {'$schema': 'http://json-schema.org/draft-07/schema#', 'title': 'ExampleModel', 'type': 'object'}

def test_schema_json():
    schema = ExampleModel.schema_json()
    assert schema == '{"$schema":"http://json-schema.org/draft-07/schema#","title":"ExampleModel","type":"object","required":["name","number"],"properties":{"name":{"type":"string"},"number":{"type":"integer","format":"int64"}}}'

def test_nested_model():
    model1 = ExampleModel(name="foo", number=3)
    model2 = ExampleModel2(model=model1)

    assert model2.model == model1

def test_nested_model_dict():
    model1 = ExampleModel(name="foo", number=3)
    model2 = ExampleModel2(model=model1)

    assert model2.dict() == {"model": {"name": "foo", "number": 3}}

def test_nested_model_json():
    model1 = ExampleModel(name="foo", number=3)
    model2 = ExampleModel2(model=model1)

    assert model2.json() == '{"model":{"name":"foo","number":3}}'

def test_pydantic_nesting():
    model1 = ExampleModel(name="foo", number=11)
    model2 = ExampleModel2(model=model1)
    model3 = ExampleModel3(model1=model1, model2=model2)

    assert model3.model1 == model1
    assert model3.model2 == model2
