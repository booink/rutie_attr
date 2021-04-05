# frozen_string_literal: true
#
require 'spec_helper'

RSpec.describe Foo do
  it "empty foo created." do
    expect(Foo.new).not_to be nil
  end

  it "test singleton method" do
    a = Foo.test?("test a")
    expect(a).to eq "test a"
  end

  it "test singleton method with default" do
    a = Foo.test?
    expect(a).to eq "a"
  end

  it "hoge instance method" do
    foo = Foo.new
    foo.foo1 = "hoge 1"
    foo.foo2 = 100
    expect(foo._hoge!("test b")).to eq "test b"
  end

  it "hoge instance method with default" do
    foo = Foo.new
    foo.foo1 = "hoge 1"
    foo.foo2 = 100
    expect(foo._hoge!).to eq "-112"
  end
end
