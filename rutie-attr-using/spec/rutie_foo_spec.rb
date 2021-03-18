# frozen_string_literal: true
#
require 'spec_helper'

RSpec.describe RutieFoo do
  it "empty foo created." do
    expect(RutieFoo.new).not_to be nil
  end
end
